use crate::io::{Decode, DecodeError, Encode};
use crate::visit::Visit;

macro_rules! def_float {
    ($ty:ident) => {
        impl Encode for $ty {
            fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
                w.write_all(&self.to_le_bytes())
            }
        }

        impl Decode for $ty {
            fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
                let mut bytes = [0; std::mem::size_of::<Self>()];
                r.read_exact(&mut bytes)?;
                Ok($ty::from_le_bytes(bytes))
            }
        }

        impl Visit for $ty {}
    };
}

def_float!(f32);
def_float!(f64);
