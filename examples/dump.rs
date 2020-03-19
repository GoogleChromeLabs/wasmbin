use std::convert::TryFrom;
use std::fs::File;
use std::io::{BufReader, Seek, SeekFrom};
use wasmbin::{module::Module, WasmbinDecode};

fn main() {
    let f = File::open(std::env::args().nth(1).expect("expected filename")).unwrap();
    let mut f = BufReader::new(f);
    let m = Module::decode(&mut f).unwrap_or_else(|err| {
        panic!(
            "Parsing error at offset 0x{:08X}: {}",
            f.seek(SeekFrom::Current(0)).unwrap(),
            err
        )
    });
    println!("{:#?}", m);
    println!("---");
    let m = wasmbin::typed_module::Module::try_from(m).unwrap();
    println!("{:#?}", m);
}
