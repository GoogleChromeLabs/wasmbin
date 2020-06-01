#[cfg(feature = "lazy-blob")]
compile_error!("Tests must be run without lazy blobs.");

use libtest_mimic::{run_tests, Arguments, Outcome, Test};
use std::error::Error;
use std::fs::{read, read_dir};
use std::path::Path;
use wabt::script::{Command, CommandKind, ScriptParser};
use wasmbin::io::Decode;
use wasmbin::module::Module;

fn fmt_wabt_error(err: wabt::Error) -> String {
    use wabt::ErrorKind;

    match err.kind() {
        ErrorKind::Nul => "string contained nul-byte".to_owned(),
        ErrorKind::Deserialize(err) => format!("failed to deserialize:\n{:#}", err),
        ErrorKind::Parse(err) => format!("failed to parse:\n{:#}", err),
        ErrorKind::WriteText => "failed to write text".to_owned(),
        ErrorKind::NonUtf8Result => "result is not a valid utf8".to_owned(),
        ErrorKind::WriteBinary => "failed to write binary".to_owned(),
        ErrorKind::ResolveNames(err) => format!("failed to resolve names:\n{:#}", err),
        ErrorKind::Validate(err) => format!("failed to validate:\n{:#}", err),
    }
}

fn fmt_wabt_script_error(err: wabt::script::Error) -> String {
    match err {
        wabt::script::Error::IoError(err) => format!("{}", err),
        wabt::script::Error::WabtError(err) => format!("Wabt: {}", fmt_wabt_error(err)),
        wabt::script::Error::Other(err) => format!("{}", err),
        wabt::script::Error::WithLineInfo { line, error } => {
            format!("at line {}: {}", line, fmt_wabt_script_error(*error))
        }
    }
}

struct WasmTest {
    module: Vec<u8>,
    expect_result: Result<(), String>,
}

fn read_tests(path: &Path, dest: &mut Vec<Test<WasmTest>>) -> Result<(), Box<dyn Error>> {
    let src = read(path)?;
    let mut features = wabt::Features::new();
    features.enable_all();
    let mut parser =
        ScriptParser::<f32, f64>::from_source_and_name_with_features(&src, "test.wast", features)
            .map_err(fmt_wabt_script_error)?;
    while let Some(Command { kind, line }) = parser.next().map_err(fmt_wabt_script_error)? {
        let (module, expect_result) = match kind {
            CommandKind::AssertMalformed { module, message } => (module, Err(message)),
            CommandKind::Module { module, .. } => (module, Ok(())),
            _ => {
                // Skipping interpreted
                continue;
            }
        };
        dest.push(Test {
            name: format!("{}:{}", path.display(), line),
            kind: String::default(),
            is_ignored: false,
            is_bench: false,
            data: WasmTest {
                module: module.into_vec(),
                expect_result,
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
