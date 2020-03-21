#![no_main]
use libfuzzer_sys::fuzz_target;

use wasmbin::{module::Module, WasmbinEncode, WasmbinDecode};

fuzz_target!(|data: &[u8]| {
    // Just check that we don't crash anywhere trying to read the data.
    let _ = Module::decode(&mut data);
});
