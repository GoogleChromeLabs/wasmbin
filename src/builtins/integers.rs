use crate::io::{DecodeError, WasmbinDecode, WasmbinEncode};
use crate::visit::WasmbinVisit;
use std::convert::TryFrom;

impl WasmbinEncode for u8 {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        std::slice::from_ref(self).encode(w)
    }
}

impl WasmbinEncode for [u8] {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        w.write_all(self)
    }
}

impl WasmbinDecode for u8 {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        let mut dest = 0;
        r.read_exact(std::slice::from_mut(&mut dest))?;
        Ok(dest)
    }
}

impl WasmbinDecode for Vec<u8> {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        let mut dest = Vec::new();
        r.read_to_end(&mut dest)?;
        Ok(dest)
    }
}

impl WasmbinVisit for u8 {}

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

impl WasmbinVisit for u32 {}

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

impl WasmbinVisit for i32 {}

impl WasmbinEncode for usize {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        match u32::try_from(*self) {
            Ok(v) => v.encode(w),
            Err(err) => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, err)),
        }
    }
}

impl WasmbinDecode for usize {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        usize::try_from(u32::decode(r)?)
            .map_err(|_| DecodeError::Leb128(leb128::read::Error::Overflow))
    }
}

impl WasmbinVisit for usize {}

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

impl WasmbinVisit for u64 {}

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

impl WasmbinVisit for i64 {}
