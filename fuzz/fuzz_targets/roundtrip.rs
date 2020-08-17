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

use wasmbin::Module;

fuzz_target!(|module: Module| {
    // Check that we don't fail to encode arbitrary Modules.
    // We're using Vec as I/O destination, so this should never fail.
    let encoded = module.encode_into(Vec::new()).unwrap();
    // Check that we can re-decoded encoded data back.
    let decoded = Module::decode_from(encoded.as_slice()).unwrap();
    // Ensure that re-decoded module is equivalent to the original.
    assert_eq!(module, decoded);
    // Check that encoding again results in a deterministic output.
    let encoded2 = decoded.encode_into(Vec::new()).unwrap();
    assert_eq!(encoded, encoded2);
});
