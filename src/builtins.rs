use super::{DecodeError, Wasmbin, WasmbinDecode, WasmbinEncode};
use std::convert::TryFrom;

impl<T: WasmbinEncode> WasmbinEncode for [T] {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        self.len().encode(w)?;
        T::encode_seq(self, w)
    }
}

impl<T: WasmbinEncode> WasmbinEncode for Vec<T> {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        self.as_slice().encode(w)
    }
}

impl<T: WasmbinDecode> WasmbinDecode for Vec<T> {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        let count = usize::decode(r)?;
        T::decode_seq(count, r)
    }
}

impl WasmbinEncode for u8 {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        Self::encode_seq(&[*self], w)
    }

    fn encode_seq(seq: &[u8], w: &mut impl std::io::Write) -> std::io::Result<()> {
        w.write_all(seq)
    }
}

impl WasmbinDecode for u8 {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        let mut dest = [0];
        r.read_exact(&mut dest)?;
        Ok(dest[0])
    }

    fn decode_seq(count: usize, r: &mut impl std::io::Read) -> Result<Vec<Self>, DecodeError> {
        let mut dest = vec![0; count];
        r.read_exact(&mut dest)?;
        Ok(dest)
    }
}

#[derive(Wasmbin)]
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

impl WasmbinEncode for u32 {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        leb128::write::unsigned(w, u64::from(*self)).map(|_| ())
    }
}

impl WasmbinDecode for u32 {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        u32::try_from(u64::decode(r)?)
            .map_err(|_| DecodeError::Leb128(leb128::read::Error::Overflow))
    }
}

impl WasmbinEncode for i32 {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        i64::from(*self).encode(w)
    }
}

impl WasmbinDecode for i32 {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        i32::try_from(i64::decode(r)?)
            .map_err(|_| DecodeError::Leb128(leb128::read::Error::Overflow))
    }
}

impl WasmbinEncode for u64 {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        leb128::write::unsigned(w, *self).map(|_| ())
    }
}

impl WasmbinDecode for u64 {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        Ok(leb128::read::unsigned(r)?)
    }
}

impl WasmbinEncode for i64 {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        leb128::write::signed(w, *self).map(|_| ())
    }
}

impl WasmbinDecode for i64 {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        Ok(leb128::read::signed(r)?)
    }
}

impl WasmbinEncode for usize {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        u32::try_from(*self).unwrap().encode(w)
    }
}

impl WasmbinDecode for usize {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        Ok(usize::try_from(u32::decode(r)?).unwrap())
    }
}

impl WasmbinEncode for str {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        self.as_bytes().encode(w)
    }
}

impl WasmbinEncode for String {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        self.as_str().encode(w)
    }
}

impl WasmbinDecode for String {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        Ok(String::from_utf8(<Vec<u8>>::decode(r)?)?)
    }
}

impl WasmbinEncode for f32 {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        w.write_all(&self.to_bits().to_le_bytes())
    }
}

impl WasmbinDecode for f32 {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        let mut bytes = [0; 4];
        r.read_exact(&mut bytes)?;
        Ok(f32::from_bits(u32::from_le_bytes(bytes)))
    }
}

impl WasmbinEncode for f64 {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        w.write_all(&self.to_bits().to_le_bytes())
    }
}

impl WasmbinDecode for f64 {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        let mut bytes = [0; 8];
        r.read_exact(&mut bytes)?;
        Ok(f64::from_bits(u64::from_le_bytes(bytes)))
    }
}
