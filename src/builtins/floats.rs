use crate::io::{Decode, DecodeError, Encode};
use crate::visit::Visit;

impl Encode for f32 {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        w.write_all(&self.to_bits().to_le_bytes())
    }
}

impl Decode for f32 {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        let mut bytes = [0; 4];
        r.read_exact(&mut bytes)?;
        Ok(f32::from_bits(u32::from_le_bytes(bytes)))
    }
}

impl Visit for f32 {}

impl Encode for f64 {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        w.write_all(&self.to_bits().to_le_bytes())
    }
}

impl Decode for f64 {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        let mut bytes = [0; std::mem::size_of::<f64>()];
        r.read_exact(&mut bytes)?;
        Ok(f64::from_bits(u64::from_le_bytes(bytes)))
    }
}

impl Visit for f64 {}
