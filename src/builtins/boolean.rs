use crate::{DecodeError, Wasmbin, WasmbinDecode, WasmbinEncode};

#[derive(Wasmbin)]
#[repr(u8)]
enum BoolRepr {
    False = 0x00,
    True = 0x01,
}

impl WasmbinEncode for bool {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        match *self {
            false => BoolRepr::False,
            true => BoolRepr::True,
        }
        .encode(w)
    }
}

impl WasmbinDecode for bool {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        Ok(match BoolRepr::decode(r)? {
            BoolRepr::False => false,
            BoolRepr::True => true,
        })
    }
}
