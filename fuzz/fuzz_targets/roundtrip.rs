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

#![no_main]
use libfuzzer_sys::fuzz_target;

use wasmbin::io::DecodeError;
use wasmbin::visit::{Visit, VisitError};
use wasmbin::Module;

// Decode and actually visit all the lazy fields to trigger decoding errors if any.
// Visit mutably to ensure that re-encoding doesn't simply reuse original raw bytes.
fn decode_and_unlazify(bytes: &[u8]) -> Result<Module, DecodeError> {
    let mut wasm = Module::decode_from(bytes)?;
    match wasm.visit_mut(|()| {}) {
        Ok(()) => Ok(wasm),
        Err(err) => match err {
            VisitError::LazyDecode(err) => Err(err),
            VisitError::Custom(err) => match err {},
        },
    }
}

enum PrintErr {}

impl<E: std::error::Error> From<E> for PrintErr {
    fn from(err: E) -> Self {
        panic!("{}", err)
    }
}

fn try_roundtrip(wasm_smith_bytes: &[u8]) -> Result<(), PrintErr> {
    // Check that we can re-decoded encoded data back.
    let my_module = decode_and_unlazify(wasm_smith_bytes)?;
    // Re-encode with `wasmbin`.
    let my_bytes = my_module.encode_into(Vec::new())?;
    // wasm-smith and wasmbin are not guaranteed to produce same bytes.
    // Instead, decode the module once again.
    let my_module_roundtrip = decode_and_unlazify(&my_bytes)?;
    // Ensure that re-decoded module is equivalent to the original.
    assert_eq!(my_module, my_module_roundtrip);
    Ok(())
}

fuzz_target!(|module: wasm_smith::Module| {
    let wasm_smith_bytes = module.to_bytes();
    try_roundtrip(&wasm_smith_bytes).unwrap_or_else(|err| match err {});
});
