#[cfg(feature = "lazy-blob")]
compile_error!("Tests must be run without lazy blobs.");

use libtest_mimic::{run_tests, Arguments, Outcome, Test};
use std::error::Error;
use std::fs::{read_dir, read_to_string};
use std::path::Path;
use wasmbin::io::Decode;
use wasmbin::module::Module;
use wast::parser::{parse, ParseBuffer};
use wast::Wast;

struct WasmTest {
    module: Vec<u8>,
    expect_result: Result<(), String>,
}

fn read_tests(path: &Path, dest: &mut Vec<Test<WasmTest>>) -> Result<(), Box<dyn Error>> {
    let src = read_to_string(path)?;
    let set_err_path_text = |mut err: wast::Error| {
        err.set_path(path);
        err.set_text(&src);
        err
    };
    let buf = ParseBuffer::new(&src).map_err(&set_err_path_text)?;
    let wast = parse::<Wast>(&buf).map_err(&set_err_path_text)?;
    for directive in wast.directives {
        let (span, mut module, expect_result) = match directive {
            wast::WastDirective::AssertMalformed {
                span,
                module: wast::QuoteModule::Module(module),
                message,
            } => (span, module, Err(message)),
            wast::WastDirective::Module(module) => (module.span, module, Ok(())),
            _ => {
                // Skipping interpreted
                continue;
            }
        };
        let (line, col) = span.linecol_in(&src);
        dest.push(Test {
            name: format!("{}:{}:{}", path.display(), line + 1, col + 1),
            kind: String::default(),
            is_ignored: match expect_result {
                // Our low-level parser doesn't validate sections.
                Err("function and code section have inconsistent lengths") => true,
                // We intentionally read future table/memory IDs.
                Err("zero flag expected") => true,
                _ => false,
            },
            is_bench: false,
            data: WasmTest {
                module: module.encode()?,
                expect_result: expect_result.map_err(|err| err.to_owned()),
            },
        });
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut tests = Vec::new();
    for file in read_dir(Path::new("tests").join("testsuite"))? {
        let path = file?.path();
        if !path.extension().map_or(false, |ext| ext == "wast") {
            continue;
        }
        if let Err(err) = read_tests(&path, &mut tests) {
            // If an error comes from the Wabt, ignore for now - most likely it uses a new feature we don't support yet.
            eprintln!("Could not read test {}: {}", path.display(), err);
        }
    }
    assert!(
        !tests.is_empty(),
        "Couldn't find any tests. Did you run `git submodule update --init`?"
    );
    run_tests(&Arguments::from_args(), tests, |test| {
        match (
            Module::decode(&mut test.data.module.as_slice()),
            &test.data.expect_result,
        ) {
            (Ok(_), Err(err)) => Outcome::Failed {
                msg: Some(format!(
                    "Expected an invalid module definition with an error: {}",
                    err
                )),
            },
            (Err(err), Ok(())) => Outcome::Failed {
                msg: Some(format!(
                    "Expected a valid module definition, but got an error:\n{:#}",
                    err
                )),
            },
            _ => Outcome::Passed,
        }
    })
    .exit()
}
