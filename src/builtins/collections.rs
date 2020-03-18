use crate::{DecodeError, WasmbinDecode, WasmbinEncode};

pub trait WasmbinCountable {}

impl<T: WasmbinCountable + WasmbinEncode> WasmbinEncode for [T] {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        self.len().encode(w)?;
        for item in self {
            item.encode(w)?;
        }
        Ok(())
    }
}

impl<T> WasmbinEncode for Vec<T> where [T]: WasmbinEncode {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        self.as_slice().encode(w)
    }
}

impl<T: WasmbinCountable + WasmbinDecode> WasmbinDecode for Vec<T> {
    fn decode(r: &mut impl std::io::BufRead) -> Result<Self, DecodeError> {
        let count = usize::decode(r)?;
        std::iter::repeat_with(|| T::decode(r)).take(count).collect()
    }
}
