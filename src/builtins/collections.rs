use crate::io::{Decode, DecodeError, Encode};

pub use wasmbin_derive::WasmbinCountable;
pub trait WasmbinCountable {}

impl<T: WasmbinCountable + Encode> Encode for [T] {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        self.len().encode(w)?;
        for item in self {
            item.encode(w)?;
        }
        Ok(())
    }
}

impl<T> Encode for Vec<T>
where
    [T]: Encode,
{
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        self.as_slice().encode(w)
    }
}

impl<T: WasmbinCountable + Decode> Decode for Vec<T> {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        let count = usize::decode(r)?;
        std::iter::repeat_with(|| T::decode(r))
            .take(count)
            .collect()
    }
}

impl_visit_for_iter!(Vec<T>);
impl_visit_for_iter!(Option<T>);
