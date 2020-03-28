use super::RawBlob;
use crate::io::{Decode, DecodeError, Encode};
use crate::visit::Visit;

impl Encode for str {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        RawBlob { contents: self }.encode(w)
    }
}

impl Encode for String {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        self.as_str().encode(w)
    }
}

impl Decode for String {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        Ok(String::from_utf8(RawBlob::decode(r)?.contents)?)
    }
}

impl Visit for String {}
