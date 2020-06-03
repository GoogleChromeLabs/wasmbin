use libtest_mimic::{run_tests, Arguments, Outcome, Test};
use std::error::Error;
use std::fs::{read_dir, read_to_string};
use std::path::Path;
use wasmbin::Module;
use wast::parser::{parse, ParseBuffer};
use wast::Wast;

const IGNORED_ERRORS: &[&str] = &[
    // We don't perform cross-section analysis.
    "function and code section have inconsistent lengths",
    // This error is actually from a test that checks for duplicate section.
    // Same as above, we don't perform cross-section analysis.
    "junk after last section",
    // We allow non-zero table and memory IDs already.
    "zero flag expected",
    // We don't perform full function analysis either.
    "too many locals",
];

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
    let buf = ParseBuffer::new(&src).map_err(set_err_path_text)?;
    let wast = parse::<Wast>(&buf).map_err(set_err_path_text)?;
    for directive in wast.directives {
        let (span, mut module, expect_result) = match directive {
            // Expect errors for assert_malformed on binary or AST modules.
            wast::WastDirective::AssertMalformed {
                span,
                module: wast::QuoteModule::Module(module),
                message,
            } => (span, module, Err(message)),
            // Expect successful parsing for regular AST modules.
            wast::WastDirective::Module(module) => (module.span, module, Ok(())),
            // Counter-intuitively, expect successful parsing for modules that are supposed
            // to error out at runtime or linking stage, too.
            wast::WastDirective::AssertInvalid { span, module, .. }
            | wast::WastDirective::AssertUnlinkable { span, module, .. } => (span, module, Ok(())),
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
                Ok(()) => false,
                Err(err) => cfg!(feature = "lazy-blob") || IGNORED_ERRORS.contains(&err),
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

fn run_test(test: &WasmTest) -> Result<(), Box<dyn Error>> {
    let module = match (
        Module::decode_from(test.module.as_slice()),
        &test.expect_result,
    ) {
        (Ok(_), Err(err)) => {
            return Err(format!(
                "Expected an invalid module definition with an error: {}",
                err
            )
            .into());
        }
        (Err(err), Ok(())) => {
            return Err(format!(
                "Expected a valid module definition, but got an error:\n{:#}",
                err
            )
            .into());
        }
        (Ok(module), Ok(())) => module,
        (Err(_), Err(_)) => return Ok(()),
    };
    let mut out = Vec::new();
    module.encode_into(&mut out)?;
    if out != test.module {
        // In the rare case that binary representation doesn't match, it
        // might be because the test uses longer LEB128 form than
        // required. Verify that at least decoding it back produces the
        // same module.
        let module2 = Module::decode_from(out.as_slice())?;
        if module != module2 {
            return Err(format!(
                "Roundtrip mismatch. Old: {:#?}\nNew: {:#?}",
                module, module2
            )
            .into());
        }
    }
    Ok(())
}

fn main() {
    if cfg!(feature = "lazy-blob") {
        eprintln!("Warning: tests are being run in a lazy mode and will be incomplete.");
    }

    let mut tests = Vec::new();
    read_dir(Path::new("tests").join("testsuite"))
        .expect("could not read the testsuite directory")
        .map(|file| {
            file.expect("could not iterate over the testsuite directory")
                .path()
        })
        .filter(|path| path.extension().map_or(false, |ext| ext == "wast"))
        .for_each(|path| {
            read_tests(&path, &mut tests).unwrap_or_else(|err| {
                panic!("Could not read test {}: {}", path.display(), err);
            });
        });
    assert!(
        !tests.is_empty(),
        "Couldn't find any tests. Did you run `git submodule update --init`?"
    );
    run_tests(&Arguments::from_args(), tests, |test| {
        match run_test(&test.data) {
            Ok(()) => Outcome::Passed,
            Err(err) => Outcome::Failed {
                msg: Some(err.to_string()),
            },
        }
    })
    .exit()
}
