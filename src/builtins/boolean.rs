use crate::io::DecodeError;
use crate::visit::Visit;

encode_decode_as!(bool, {
    false <=> 0_u8,
    true <=> 1_u8,
}, |discriminant| {
    Err(DecodeError::UnsupportedDiscriminant { ty: "bool", discriminant: discriminant.into() })
});

impl Visit for bool {}
