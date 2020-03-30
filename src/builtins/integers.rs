use crate::io::{Decode, DecodeError, DecodeWithDiscriminant, Encode};
use crate::visit::Visit;
use std::convert::TryFrom;

impl Encode for u8 {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        std::slice::from_ref(self).encode(w)
    }
}

impl Encode for [u8] {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        w.write_all(self)
    }
}

impl Decode for u8 {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        let mut dest = 0;
        r.read_exact(std::slice::from_mut(&mut dest))?;
        Ok(dest)
    }
}

impl Decode for Option<u8> {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        let mut dest = 0;
        loop {
            return match r.read(std::slice::from_mut(&mut dest)) {
                Ok(0) => Ok(None),
                Ok(_) => Ok(Some(dest)),
                Err(err) if err.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(err) => Err(DecodeError::Io(err)),
            };
        }
    }
}

impl DecodeWithDiscriminant for u8 {
    fn maybe_decode_with_discriminant(
        discriminant: u8,
        _r: &mut impl std::io::Read,
    ) -> std::result::Result<std::option::Option<Self>, DecodeError> {
        Ok(Some(discriminant))
    }
}

impl Decode for Vec<u8> {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        let mut dest = Vec::new();
        r.read_to_end(&mut dest)?;
        Ok(dest)
    }
}

impl Visit for u8 {}

impl Encode for u32 {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        leb128::write::unsigned(w, u64::from(*self)).map(|_| ())
    }
}

impl Decode for u32 {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        u32::try_from(u64::decode(r)?)
            .map_err(|_| DecodeError::Leb128(leb128::read::Error::Overflow))
    }
}

impl Visit for u32 {}

impl Encode for i32 {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        i64::from(*self).encode(w)
    }
}

impl Decode for i32 {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        i32::try_from(i64::decode(r)?)
            .map_err(|_| DecodeError::Leb128(leb128::read::Error::Overflow))
    }
}

impl Visit for i32 {}

impl Encode for usize {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        match u32::try_from(*self) {
            Ok(v) => v.encode(w),
            Err(err) => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, err)),
        }
    }
}

impl Decode for usize {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        usize::try_from(u32::decode(r)?)
            .map_err(|_| DecodeError::Leb128(leb128::read::Error::Overflow))
    }
}

impl Visit for usize {}

impl Encode for u64 {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        leb128::write::unsigned(w, *self).map(|_| ())
    }
}

impl Decode for u64 {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        Ok(leb128::read::unsigned(r)?)
    }
}

impl Visit for u64 {}

impl Encode for i64 {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        leb128::write::signed(w, *self).map(|_| ())
    }
}

impl Decode for i64 {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        Ok(leb128::read::signed(r)?)
    }
}

impl Visit for i64 {}
