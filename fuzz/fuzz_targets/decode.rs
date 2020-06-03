#![no_main]
use libfuzzer_sys::fuzz_target;

use wasmbin::Module;

fuzz_target!(|data: &[u8]| {
    // Just check that we don't crash anywhere trying to read the data.
    let _ = Module::decode_from(data);
});
