use std::fs::File;
use std::io::{BufReader, Seek, SeekFrom};
use wasmbin::Module;

fn main() {
    let f = File::open(std::env::args().nth(1).expect("expected filename")).unwrap();
    let mut f = BufReader::new(f);
    let m = Module::decode_from(&mut f).unwrap_or_else(|err| {
        panic!(
            "Parsing error at offset 0x{:08X}: {}",
            f.seek(SeekFrom::Current(0)).unwrap(),
            err
        )
    });
    println!("{:#?}", m);
}
