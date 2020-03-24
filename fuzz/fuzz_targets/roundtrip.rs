#![no_main]
use libfuzzer_sys::fuzz_target;

use wasmbin::module::Module;
use wasmbin::io::{WasmbinEncode, WasmbinDecode};

fuzz_target!(|module: Module| {
    let mut encoded = Vec::new();
    // Check that we don't fail to encode arbitrary Modules.
    // We're using Vec as I/O destination, so this should never fail.
    module.encode(&mut encoded).unwrap();
    // Check that we can re-decoded encoded data back.
    let decoded = Module::decode(&mut encoded.as_slice()).unwrap();
    // Ensure that re-decoded module is equivalent to the original.
    assert_eq!(module, decoded);
    // Check that encoding again results in a deterministic output.
    let mut encoded2 = Vec::new();
    decoded.encode(&mut encoded2).unwrap();
    assert_eq!(encoded, encoded2);
});
