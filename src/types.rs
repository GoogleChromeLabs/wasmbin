use crate::{
    wasmbin_discriminants, DecodeError, Wasmbin, WasmbinCountable, WasmbinDecode, WasmbinEncode,
};
use arbitrary::Arbitrary;
use std::fmt::{self, Debug, Formatter};

#[derive(Wasmbin, WasmbinCountable, Debug, Arbitrary, PartialEq, Eq)]
pub enum ValueType {
    I32 = 0x7F,
    I64 = 0x7E,
    F32 = 0x7D,
    F64 = 0x7C,
}

#[wasmbin_discriminants]
#[derive(Wasmbin, Debug, Arbitrary, PartialEq, Eq)]
pub enum BlockType {
    Empty = 0x40,
    Value(ValueType),
}

#[derive(Wasmbin, WasmbinCountable, Arbitrary, PartialEq, Eq)]
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

#[derive(Arbitrary, PartialEq, Eq)]
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

#[derive(Wasmbin)]
enum LimitsRepr {
    #[wasmbin(discriminant = 0x00)]
    Min { min: u32 },

    #[wasmbin(discriminant = 0x01)]
    MinMax { min: u32, max: u32 },
}

impl WasmbinEncode for Limits {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        let min = self.min;
        match self.max {
            Some(max) => LimitsRepr::MinMax { min, max },
            None => LimitsRepr::Min { min },
        }
        .encode(w)
    }
}

impl WasmbinDecode for Limits {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        Ok(match LimitsRepr::decode(r)? {
            LimitsRepr::Min { min } => Limits { min, max: None },
            LimitsRepr::MinMax { min, max } => Limits {
                min,
                max: Some(max),
            },
        })
    }
}

#[derive(Wasmbin, WasmbinCountable, Debug, Arbitrary, PartialEq, Eq)]
pub struct MemType {
    pub limits: Limits,
}

#[derive(Wasmbin, Debug, Arbitrary, PartialEq, Eq)]
pub enum ElemType {
    FuncRef = 0x70,
}

#[derive(Wasmbin, WasmbinCountable, Debug, Arbitrary, PartialEq, Eq)]
pub struct TableType {
    pub elem_type: ElemType,
    pub limits: Limits,
}

#[derive(Wasmbin, Debug, Arbitrary, PartialEq, Eq)]
pub struct GlobalType {
    pub value_type: ValueType,
    pub mutable: bool,
}
