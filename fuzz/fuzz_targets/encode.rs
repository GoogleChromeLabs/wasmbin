#![no_main]
use libfuzzer_sys::fuzz_target;

use wasmbin::{module::Module, WasmbinEncode, WasmbinDecode};

fuzz_target!(|module: Module| {
    let mut encoded = Vec::new();
    module.encode(&mut encoded).unwrap();
    let decoded = Module::decode(&mut encoded.as_slice()).unwrap();
    assert_eq!(module, decoded);
});
