use super::{DecodeError, Wasmbin, WasmbinDecode, WasmbinEncode};
use std::convert::TryFrom;

#[repr(transparent)]
pub struct Blob<T>(pub T);

impl<T> std::ops::Deref for Blob<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> std::ops::DerefMut for Blob<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: WasmbinEncode> WasmbinEncode for Blob<T> {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        let mut dest = Vec::new();
        self.0.encode(&mut dest)?;
        dest.len().encode(w)?;
        w.write_all(&dest)
    }
}

impl<T: WasmbinDecode> WasmbinDecode for Blob<T> {
    fn decode(r: &mut impl std::io::BufRead) -> Result<Self, DecodeError> {
        let size = u32::decode(r)?;
        let mut taken = std::io::Read::take(r, size.into());
        let value = T::decode(&mut taken)?;
        if taken.limit() != 0 {
            return Err(DecodeError::UnrecognizedData);
        }
        Ok(Blob(value))
    }
}

impl<T: WasmbinEncode> WasmbinEncode for [T] {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        T::encode_seq(self, w)
    }
}

impl<T> WasmbinEncode for Vec<T> where [T]: WasmbinEncode {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        self.as_slice().encode(w)
    }
}

impl<T: WasmbinDecode> WasmbinDecode for Vec<T> {
    fn decode(r: &mut impl std::io::BufRead) -> Result<Self, DecodeError> {
        T::decode_seq(r)
    }
}

impl WasmbinEncode for u8 {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        std::slice::from_ref(self).encode(w)
    }

    fn encode_seq(seq: &[u8], w: &mut impl std::io::Write) -> std::io::Result<()> {
        w.write_all(seq)
    }
}

impl WasmbinDecode for u8 {
    fn decode(r: &mut impl std::io::BufRead) -> Result<Self, DecodeError> {
        let mut dest = 0;
        r.read_exact(std::slice::from_mut(&mut dest))?;
        Ok(dest)
    }

    fn decode_seq(r: &mut impl std::io::BufRead) -> Result<Vec<Self>, DecodeError> {
        let mut dest = Vec::new();
        r.read_to_end(&mut dest)?;
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
    fn decode(r: &mut impl std::io::BufRead) -> Result<Self, DecodeError> {
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
    fn decode(r: &mut impl std::io::BufRead) -> Result<Self, DecodeError> {
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
    fn decode(r: &mut impl std::io::BufRead) -> Result<Self, DecodeError> {
        i32::try_from(i64::decode(r)?)
            .map_err(|_| DecodeError::Leb128(leb128::read::Error::Overflow))
    }
}

impl WasmbinEncode for usize {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        u32::try_from(*self).unwrap().encode(w)
    }
}

impl WasmbinDecode for usize {
    fn decode(r: &mut impl std::io::BufRead) -> Result<Self, DecodeError> {
        Ok(usize::try_from(u32::decode(r)?).unwrap())
    }
}

impl WasmbinEncode for u64 {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        leb128::write::unsigned(w, *self).map(|_| ())
    }
}

impl WasmbinDecode for u64 {
    fn decode(r: &mut impl std::io::BufRead) -> Result<Self, DecodeError> {
        Ok(leb128::read::unsigned(r)?)
    }
}

impl WasmbinEncode for i64 {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        leb128::write::signed(w, *self).map(|_| ())
    }
}

impl WasmbinDecode for i64 {
    fn decode(r: &mut impl std::io::BufRead) -> Result<Self, DecodeError> {
        Ok(leb128::read::signed(r)?)
    }
}

impl WasmbinEncode for str {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        Blob(self.as_bytes()).encode(w)
    }
}

impl WasmbinEncode for String {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        self.as_str().encode(w)
    }
}

impl WasmbinDecode for String {
    fn decode(r: &mut impl std::io::BufRead) -> Result<Self, DecodeError> {
        Ok(String::from_utf8(Blob::decode(r)?.0)?)
    }
}

impl WasmbinEncode for f32 {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        w.write_all(&self.to_bits().to_le_bytes())
    }
}

impl WasmbinDecode for f32 {
    fn decode(r: &mut impl std::io::BufRead) -> Result<Self, DecodeError> {
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
    fn decode(r: &mut impl std::io::BufRead) -> Result<Self, DecodeError> {
        let mut bytes = [0; 8];
        r.read_exact(&mut bytes)?;
        Ok(f64::from_bits(u64::from_le_bytes(bytes)))
    }
}
