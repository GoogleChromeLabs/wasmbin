// Copyright 2020 Google Inc. All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use anyhow::{bail, Error};
use fehler::throws;
use libtest_mimic::{run_tests, Arguments, Outcome, Test};
use std::fs::{read_dir, read_to_string};
use std::path::Path;
use wasmbin::Module;
use wast::parser::{parse, ParseBuffer};
use wast::Wast;

const IGNORED_ERRORS: &[&str] = &[
    // We don't perform cross-section analysis.
    "function and code section have inconsistent lengths",
    "data count section required",
    "data count and data section have inconsistent lengths",
    // We allow non-zero table and memory IDs already.
    "zero flag expected",
    // We don't perform full function analysis either.
    "too many locals",
];

struct WasmTest {
    module: Vec<u8>,
    expect_result: Result<(), String>,
}

#[throws]
fn read_tests_from_file(path: &Path, dest: &mut Vec<Test<WasmTest>>) {
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
            }
            // Unlike other AssertInvalid, this is actually something we
            // check at the parsing time, because it's part of our
            // typesystem and doesn't require cross-function or
            // cross-section checks.
            | wast::WastDirective::AssertInvalid {
                span,
                module,
                message: message @ "invalid lane index",
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
}

#[throws]
fn read_tests_from_dir(path: &Path, dest: &mut Vec<Test<WasmTest>>) {
    for file in read_dir(path)? {
        let path = file?.path();
        if path.extension().map_or(false, |ext| ext == "wast") {
            read_tests_from_file(&path, dest)?;
        }
    }
}

#[throws]
fn read_all_tests(path: &Path) -> Vec<Test<WasmTest>> {
    let mut tests = Vec::new();
    read_tests_from_dir(path, &mut tests)?;
    let proposals_dir = path.join("proposals");

    macro_rules! read_proposal_tests {
        ($name:literal) => {
            if cfg!(feature = $name) {
                read_proposal_tests!(unconditional $name)
            }
        };

        (unconditional $name:literal) => {
            read_tests_from_dir(&proposals_dir.join($name), &mut tests)?
        };
    }

    read_proposal_tests!("bulk-memory-operations");
    read_proposal_tests!(unconditional "multi-value");
    read_proposal_tests!(unconditional "mutable-global");
    read_proposal_tests!(unconditional "nontrapping-float-to-int-conversions");
    read_proposal_tests!("reference-types");
    read_proposal_tests!(unconditional "sign-extension-ops");
    read_proposal_tests!("simd");
    read_proposal_tests!("tail-call");

    if tests.is_empty() {
        bail!("Couldn't find any tests. Did you run `git submodule update --init`?");
    }

    tests
}

#[throws]
fn run_test(test: &WasmTest) {
    let mut slice = test.module.as_slice();
    let module = match (Module::decode_from(&mut slice), &test.expect_result) {
        (Ok(_), Err(err)) => bail!("Expected an invalid module definition with an error: {}", err),
        (Err(err), Ok(())) => bail!(
            "Expected a valid module definition, but got an error\nParsed part: {:02X?}\nUnparsed part: {:02X?}\nError: {:#}",
            &test.module[..test.module.len() - slice.len()],
            slice,
            err
        ),
        (Ok(module), Ok(())) => module,
        (Err(_), Err(_)) => return,
    };
    let out = module.encode_into(Vec::new())?;
    if out != test.module {
        // In the rare case that binary representation doesn't match, it
        // might be because the test uses longer LEB128 form than
        // required. Verify that at least decoding it back produces the
        // same module.
        let module2 = Module::decode_from(out.as_slice())?;
        if module != module2 {
            bail!(
                "Roundtrip mismatch. Old: {:#?}\nNew: {:#?}",
                module,
                module2
            );
        }
    }
}

#[throws]
fn main() {
    if cfg!(feature = "lazy-blob") {
        eprintln!("Warning: tests are being run in a lazy mode and will be incomplete.");
    }
    let tests = read_all_tests(&Path::new("tests").join("testsuite"))?;
    run_tests(&Arguments::from_args(), tests, |test| {
        match run_test(&test.data) {
            Ok(()) => Outcome::Passed,
            Err(err) => Outcome::Failed {
                msg: Some(err.to_string()),
            },
        }
    })
    .exit_if_failed();
}
