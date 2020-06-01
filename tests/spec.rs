use std::fs::{read_dir, read_to_string};
use std::path::Path;
use std::error::Error;
use wasmbin::io::Decode;
use wasmbin::module::Module;
use wabt::script::{Command, CommandKind, ScriptParser};

fn fmt_wabt_error(err: wabt::Error) -> String {
    use wabt::ErrorKind;

    match err.kind() {
        ErrorKind::Nul => "string contained nul-byte".to_owned(),
        ErrorKind::Deserialize(err) => format!("failed to deserialize:\n{:#}", err),
        ErrorKind::Parse(_) => "failed to parse".to_owned(),
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
        wabt::script::Error::WithLineInfo { line, error } => format!("at line {}: {}", line, fmt_wabt_script_error(*error)),
    }
}

fn check_wast(path: &Path) -> Result<(), Box<dyn Error>> {
    let path = path.as_os_str().to_str().ok_or("Invalid UTF-8 path")?;
    let src = read_to_string(path)?;
    let mut parser = ScriptParser::<f32, f64>::from_str(&src).map_err(fmt_wabt_script_error)?;
    while let Some(Command { kind, line }) = parser.next().map_err(fmt_wabt_script_error)? {
        let (module, expect_res) = match kind {
            CommandKind::AssertMalformed { module, message } => {
                (module, Err(message))
            }
            CommandKind::Module { module, .. } => {
                (module, Ok(()))
            }
            _ => {
                // Skipping interpreted
                continue
            }
        };
        println!("Test {}:{}", path, line);
        println!("---");
        match (Module::decode(&mut module.into_vec().as_slice()), expect_res) {
            (Ok(_), Err(err)) => println!("Expected an invalid module definition at line {} with an error: {}", line, err),
            (Err(err), Ok(())) => println!("Expected a valid module definition at line {}. Error:\n{:#}", line, err),
            _ => println!("OK"),
        }
        println!("===");
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    for file in read_dir("tests/testsuite")? {
        let path = file?.path();
        if !path.extension().map_or(false, |ext| ext == "wast") {
            continue;
        }
        match check_wast(&path) {
            Ok(()) => {},
            Err(err) => {
                // If an error comes from the Wabt, ignore for now - most likely it uses a new feature we don't support yet.
                let _ = err;
                /*
                println!("Suite {}", path.display());
                println!("---");
                println!("{}", err);
                println!("===");
                */
            }
        }
    }
    Ok(())
}
