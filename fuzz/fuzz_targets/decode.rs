#![no_main]
use libfuzzer_sys::fuzz_target;

use wasmbin::{module::Module, WasmbinEncode, WasmbinDecode};

fuzz_target!(|orig_data: &[u8]| {
    let mut data = orig_data;
    if let Ok(decoded) = Module::decode(&mut data) {
        let mut encoded = Vec::new();
        decoded.encode(&mut encoded).unwrap();
        assert_eq!(orig_data, encoded.as_slice(), "Could not roundtrip {:#?}", decoded);
    }
});
