use crate::builtins::WasmbinCountable;
use crate::indices::TypeId;
use crate::io::{Decode, DecodeError, DecodeWithDiscriminant, Encode, Wasmbin};
use crate::visit::Visit;
use crate::wasmbin_discriminants;
use arbitrary::Arbitrary;
use std::convert::TryFrom;
use std::fmt::{self, Debug, Formatter};

const OP_CODE_EMPTY_BLOCK: u8 = 0x40;

#[cfg_attr(feature = "reference-types", wasmbin_discriminants)]
#[derive(Wasmbin, WasmbinCountable, Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
#[repr(u8)]
pub enum ValueType {
    I32 = 0x7F,
    I64 = 0x7E,
    F32 = 0x7D,
    F64 = 0x7C,
    #[cfg(feature = "simd")]
    V128 = 0x7B,
    #[cfg(feature = "reference-types")]
    Ref(RefType),
}

#[derive(Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
#[repr(u8)]
pub enum BlockType {
    Empty,
    Value(ValueType),
    MultiValue(TypeId),
}

impl Encode for BlockType {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        match self {
            BlockType::Empty => OP_CODE_EMPTY_BLOCK.encode(w),
            BlockType::Value(ty) => ty.encode(w),
            BlockType::MultiValue(id) => i64::from(id.index).encode(w),
        }
    }
}

impl Decode for BlockType {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        let discriminant = u8::decode(r)?;
        if discriminant == OP_CODE_EMPTY_BLOCK {
            return Ok(BlockType::Empty);
        }
        if let Some(ty) = ValueType::maybe_decode_with_discriminant(discriminant, r)? {
            return Ok(BlockType::Value(ty));
        }
        // We have already read one byte that could've been either a
        // discriminant or a part of an s33 LEB128 specially used for
        // type indices.
        //
        // To recover the LEB128 sequence, we need to chain it back.
        let buf = [discriminant];
        let mut r = std::io::Read::chain(&buf[..], r);
        let as_i64 = i64::decode(&mut r)?;
        // These indices are encoded as positive signed integers.
        // Convert them to unsigned integers and error out if they're out of range.
        let index = u32::try_from(as_i64).map_err(|_| leb128::read::Error::Overflow)?;
        Ok(BlockType::MultiValue(TypeId { index }))
    }
}

#[derive(Wasmbin, WasmbinCountable, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
#[wasmbin(discriminant = 0x60)]
pub struct FuncType {
    pub params: Vec<ValueType>,
    pub results: Vec<ValueType>,
}

impl Debug for FuncType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        fn encode_types(types: &[ValueType], f: &mut Formatter) -> fmt::Result {
            f.write_str("(")?;
            for (i, ty) in types.iter().enumerate() {
                if i != 0 {
                    f.write_str(", ")?;
                }
                ty.fmt(f)?;
            }
            f.write_str(")")
        }

        encode_types(&self.params, f)?;
        f.write_str(" -> ")?;
        encode_types(&self.results, f)
    }
}

#[derive(Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
pub struct Limits {
    pub min: u32,
    pub max: Option<u32>,
}

impl Debug for Limits {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}..", self.min)?;
        if let Some(max) = self.max {
            write!(f, "={}", max)?;
        }
        Ok(())
    }
}

#[wasmbin_discriminants]
#[derive(Wasmbin)]
#[repr(u8)]
enum LimitsRepr {
    Min { min: u32 } = 0x00,
    MinMax { min: u32, max: u32 } = 0x01,
}

encode_decode_as!(Limits, {
    (Limits { min, max: None }) <=> (LimitsRepr::Min { min }),
    (Limits { min, max: Some(max) }) <=> (LimitsRepr::MinMax { min, max }),
});

#[derive(Wasmbin, WasmbinCountable, Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
pub struct MemType {
    pub limits: Limits,
}

#[derive(Wasmbin, Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
#[repr(u8)]
pub enum RefType {
    Func = 0x70,
    #[cfg(feature = "reference-types")]
    Extern = 0x6F,
}

#[derive(Wasmbin, WasmbinCountable, Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
pub struct TableType {
    pub elem_type: RefType,
    pub limits: Limits,
}

#[derive(Wasmbin, Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
pub struct GlobalType {
    pub value_type: ValueType,
    pub mutable: bool,
}
