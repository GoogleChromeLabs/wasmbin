use crate::{DecodeError, WasmbinCountable, WasmbinDecode, WasmbinEncode};

#[repr(transparent)]
#[derive(Default)]
pub struct Blob<T>(pub T);

impl<T: std::fmt::Debug> std::fmt::Debug for Blob<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("Blob(")?;
        self.0.fmt(f)?;
        f.write_str(")")
    }
}

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

impl<R: ?Sized, T> AsRef<R> for Blob<T> where T: AsRef<R> {
    fn as_ref(&self) -> &R {
        self.0.as_ref()
    }
}

impl<R: ?Sized, T> AsMut<R> for Blob<T> where T: AsMut<R> {
    fn as_mut(&mut self) -> &mut R {
        self.0.as_mut()
    }
}

impl<T> std::borrow::Borrow<T> for Blob<T> {
    fn borrow(&self) -> &T {
        self
    }
}

impl<T> std::borrow::BorrowMut<T> for Blob<T> {
    fn borrow_mut(&mut self) -> &mut T {
        self
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
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        let size = u32::decode(r)?;
        let mut taken = std::io::Read::take(r, size.into());
        let value = T::decode(&mut taken)?;
        if taken.limit() != 0 {
            return Err(DecodeError::UnrecognizedData);
        }
        Ok(Blob(value))
    }
}

impl<T: WasmbinCountable> WasmbinCountable for Blob<T> {}
