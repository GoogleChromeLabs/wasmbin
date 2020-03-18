use super::Blob;
use crate::{DecodeError, WasmbinDecode, WasmbinEncode};

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
