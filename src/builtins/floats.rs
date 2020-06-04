use crate::io::{Decode, DecodeError, Encode};
use crate::visit::Visit;

macro_rules! def_float {
    ($ty:ident) => {
        impl Encode for $ty {
            fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
                self.to_le_bytes().encode(w)
            }
        }

        impl Decode for $ty {
            fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
                Decode::decode(r).map($ty::from_le_bytes)
            }
        }

        impl Visit for $ty {}
    };
}

def_float!(f32);
def_float!(f64);
