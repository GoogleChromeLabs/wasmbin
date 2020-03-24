use crate::builtins::WasmbinCountable;
use crate::io::{DecodeError, WasmbinDecode, WasmbinEncode};
use crate::visit::WasmbinVisit;
use arbitrary::Arbitrary;

#[cfg(feature = "lazy-blob")]
macro_rules! if_lazy {
    (if lazy { $($then:tt)* } $(else { $($otherwise:tt)* })?) => {
        $($then)*
    };
}

#[cfg(not(feature = "lazy-blob"))]
macro_rules! if_lazy {
    (if lazy { $($then:tt)* } $(else { $($otherwise:tt)* })?) => {
        $($($otherwise)*)?
    };
}

#[derive(Debug, Arbitrary, PartialEq, Eq, Hash, Clone, WasmbinVisit)]
pub struct RawBlob<T = Vec<u8>> {
    pub contents: T,
}

impl<T: AsRef<[u8]>> WasmbinEncode for RawBlob<T> {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        let bytes = self.contents.as_ref();
        bytes.len().encode(w)?;
        bytes.encode(w)
    }
}

impl<T: WasmbinDecode> WasmbinDecode for RawBlob<T> {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        let size = u32::decode(r)?;
        let mut taken = std::io::Read::take(r, size.into());
        let contents = T::decode(&mut taken)?;
        if taken.limit() != 0 {
            return Err(DecodeError::UnrecognizedData);
        }
        Ok(RawBlob { contents })
    }
}

impl<T: AsRef<[u8]>> AsRef<[u8]> for RawBlob<T> {
    fn as_ref(&self) -> &[u8] {
        self.contents.as_ref()
    }
}

if_lazy!(if lazy {
    use crate::builtins::Lazy as BlobContents;

    impl<T: WasmbinDecode> Blob<T> {
        pub fn try_contents(&self) -> Result<&T, DecodeError> {
            self.contents.try_contents()
        }

        pub fn try_contents_mut(&mut self) -> Result<&mut T, DecodeError> {
            self.contents.try_contents_mut()
        }

        pub fn try_into_contents(self) -> Result<T, DecodeError> {
            self.contents.try_into_contents()
        }
    }
} else {
    type BlobContents<T> = T;

    impl<T: WasmbinDecode> Blob<T> {
        pub fn try_contents(&self) -> Result<&T, DecodeError> {
            Ok(&self.contents)
        }

        pub fn try_contents_mut(&mut self) -> Result<&mut T, DecodeError> {
            Ok(&mut self.contents)
        }

        pub fn try_into_contents(self) -> Result<T, DecodeError> {
            Ok(self.contents)
        }
    }
});

#[derive(Default, Arbitrary, PartialEq, Eq, Hash, Clone, WasmbinVisit)]
pub struct Blob<T: WasmbinDecode> {
    contents: BlobContents<T>,
}

impl<T: WasmbinDecode + std::fmt::Debug> std::fmt::Debug for Blob<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("Blob(")?;
        self.contents.fmt(f)?;
        f.write_str(")")
    }
}

impl<T: WasmbinDecode + WasmbinEncode> WasmbinEncode for Blob<T> {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        let value = if_lazy!(if lazy {
            match self.contents.try_as_raw() {
                Ok(raw) => return RawBlob { contents: raw }.encode(w),
                Err(value) => value,
            }
        } else {
            &self.contents
        });
        let mut buf;
        buf = Vec::new();
        value.encode(&mut buf)?;
        RawBlob { contents: buf }.encode(w)
    }
}

impl<T: WasmbinDecode> WasmbinDecode for Blob<T> {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        Ok(Blob {
            contents: RawBlob::decode(r)?.contents,
        })
    }
}

impl<T: WasmbinDecode + WasmbinCountable> WasmbinCountable for Blob<T> {}

impl<T: WasmbinDecode> From<T> for Blob<T> {
    fn from(value: T) -> Self {
        Blob {
            contents: value.into(),
        }
    }
}
