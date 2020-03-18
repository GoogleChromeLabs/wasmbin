use crate::sections::Section;
use crate::{DecodeError, Wasmbin, WasmbinDecode, WasmbinEncode};

const MAGIC_AND_VERSION: [u8; 8] = [b'\0', b'a', b's', b'm', 0x01, 0x00, 0x00, 0x00];

#[derive(Debug, Default)]
struct MagicAndVersion;

impl WasmbinEncode for MagicAndVersion {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        w.write_all(&MAGIC_AND_VERSION)
    }
}

impl WasmbinDecode for MagicAndVersion {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        let mut magic_and_version = [0; 8];
        r.read_exact(&mut magic_and_version)?;
        if magic_and_version != MAGIC_AND_VERSION {
            return Err(DecodeError::InvalidMagic);
        }
        Ok(MagicAndVersion)
    }
}

#[derive(Wasmbin, Debug, Default)]
pub struct Module {
    magic_and_version: MagicAndVersion,
    pub sections: Vec<Section>,
}
