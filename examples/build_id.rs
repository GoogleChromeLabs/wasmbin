use std::fs::OpenOptions;
use std::io::{BufReader, BufWriter, Seek, SeekFrom};
use wasmbin::builtins::Blob;
use wasmbin::sections::{CustomSection, RawCustomSection, Section};
use wasmbin::Module;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filename = std::env::args()
        .nth(1)
        .expect("Provide a filename as an argument");
    let mut f = OpenOptions::new().read(true).write(true).open(filename)?;
    let mut module = Module::decode_from(BufReader::new(&mut f))?;
    let uuid = uuid::Uuid::new_v4();
    println!("Generated UUID: {}", uuid);
    module
        .sections
        .push(Section::Custom(Blob::from(CustomSection::Other(
            RawCustomSection {
                name: "build_id".to_string(),
                data: uuid.as_bytes().to_vec(),
            },
        ))));
    f.set_len(0)?;
    f.seek(SeekFrom::Start(0))?;
    module.encode_into(BufWriter::new(f))?;
    Ok(())
}
