use crate::lazy_mut::{LazyMut, LazyTransform};
use crate::{DecodeError, WasmbinCountable, WasmbinDecode, WasmbinEncode};

pub struct RawBlob<T: AsRef<[u8]> = Vec<u8>> {
    pub contents: T,
}

impl<T: AsRef<[u8]>> WasmbinEncode for RawBlob<T> {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        let bytes = self.contents.as_ref();
        bytes.len().encode(w)?;
        bytes.encode(w)
    }
}

impl WasmbinDecode for RawBlob {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        let size = u32::decode(r)?;
        let mut taken = std::io::Read::take(r, size.into());
        let contents = Vec::decode(&mut taken)?;
        Ok(RawBlob { contents })
    }
}

impl<T: AsRef<[u8]>> AsRef<[u8]> for RawBlob<T> {
    fn as_ref(&self) -> &[u8] {
        self.contents.as_ref()
    }
}

struct BlobTransform;

impl<T: WasmbinDecode> LazyTransform<Box<[u8]>, Result<T, DecodeError>> for BlobTransform {
    fn lazy_transform(input: &Box<[u8]>) -> Result<T, DecodeError> {
        let mut slice = input.as_ref();
        let decoded = T::decode(&mut slice)?;
        if !slice.is_empty() {
            return Err(DecodeError::UnrecognizedData);
        }
        Ok(decoded)
    }
}

#[derive(Default)]
pub struct Blob<T> {
    contents: LazyMut<Box<[u8]>, T, BlobTransform>,
}

impl<T: WasmbinDecode> Blob<T> {
    pub fn try_contents(&self) -> Result<&T, DecodeError> {
        self.contents.try_output()
    }

    pub fn try_contents_mut(&mut self) -> Result<&mut T, DecodeError> {
        self.contents.try_output_mut()
    }

    pub fn try_into_contents(self) -> Result<T, DecodeError> {
        self.contents.try_into_output()
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Blob<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("Blob(")?;
        self.contents.fmt(f)?;
        f.write_str(")")
    }
}

impl<T: WasmbinEncode> WasmbinEncode for Blob<T> {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        let mut buf;
        let buf_slice = match self.contents.input_res() {
            Ok(input) => input.as_ref(),
            Err(output) => {
                buf = Vec::new();
                output.encode(&mut buf)?;
                buf.as_slice()
            }
        };
        RawBlob {
            contents: buf_slice,
        }
        .encode(w)
    }
}

impl<T: WasmbinDecode> WasmbinDecode for Blob<T> {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        let raw = RawBlob::decode(r)?;
        Ok(Blob {
            contents: LazyMut::new(raw.contents.into_boxed_slice()),
        })
    }
}

impl<T: WasmbinCountable> WasmbinCountable for Blob<T> {}
