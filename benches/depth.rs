#[cfg(feature = "lazy-blob")]
compile_error!("Lazy blobs must be disabled for depth test.");

use std::io::{self, Cursor, Read};
use wasmbin::instructions::{BlockBody, Expression, Instruction};
use wasmbin::io::WasmbinEncode;
use wasmbin::module::MagicAndVersion;
use wasmbin::types::BlockType;

struct DeepBlock {
    buf: Cursor<Vec<u8>>,
    depth: usize,
}

impl DeepBlock {
    fn try_new() -> io::Result<Self> {
        let mut buf = Vec::new();

        MagicAndVersion.encode(&mut buf)?;
        // Code section ID
        10_u8.encode(&mut buf)?;
        // Code section size
        u32::max_value().encode(&mut buf)?;
        // Code section entries count
        1_u32.encode(&mut buf)?;
        // Function body size
        u32::max_value().encode(&mut buf)?;
        // Local groups count
        0_u32.encode(&mut buf)?;
        // ...to be continued with Expression node

        Ok(Self {
            buf: Cursor::new(buf),
            depth: 0,
        })
    }
}

impl Read for DeepBlock {
    fn read(&mut self, dest: &mut [u8]) -> io::Result<usize> {
        loop {
            match self.buf.read(dest) {
                Ok(0) => {
                    if self.depth % 10 == 0 {
                        eprintln!("Depth: {}", self.depth);
                    }
                    self.depth += 1;

                    let mut buf = Vec::new();

                    Instruction::Block(BlockBody {
                        return_type: BlockType::Empty,
                        expr: Expression::default(),
                    })
                    .encode(&mut buf)?;

                    // remove SeqInstructionRepr::End to make infinite block
                    assert_eq!(buf.pop(), Some(0x0B));

                    self.buf = Cursor::new(buf);
                }
                res => return res,
            }
        }
    }
}

#[cfg(not(any(feature = "walrus", feature = "parity-wasm")))]
fn main() {
    use wasmbin::io::WasmbinDecode;
    use wasmbin::module::Module;

    let mut deep_block = DeepBlock::try_new().unwrap();
    Module::decode(&mut deep_block).unwrap();
    unreachable!()
}

#[cfg(feature = "walrus")]
fn main() {
    let mut deep_block = DeepBlock::try_new().unwrap();
    // walrus doesn't support streaming readers, so we have to
    // create a temporary buffer with a limited size
    let mut buf = vec![0; 100_000_000];
    std::io::copy(&mut deep_block, &mut buf).unwrap();
    walrus::Module::from_buffer(&buf).unwrap();
    unreachable!()
}

#[cfg(feature = "parity-wasm")]
fn main() {
    use parity_wasm::elements::{Deserialize, Module};

    let mut deep_block = DeepBlock::try_new().unwrap();
    Module::deserialize(&mut deep_block).unwrap();
    unreachable!()
}
