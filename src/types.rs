use crate::{Wasmbin, DecodeError, WasmbinEncode, WasmbinDecode};

#[derive(Wasmbin)]
pub enum ValueType {
    I32 = 0x7F,
    I64 = 0x7E,
    F32 = 0x7D,
    F64 = 0x7C,
}

#[derive(Wasmbin)]
pub enum BlockType {
    #[wasmbin(discriminant = 0x40)]
    Empty,

    Value(ValueType),
}

#[derive(Wasmbin)]
#[wasmbin(discriminant = 0x60)]
pub struct FuncType {
    pub params: Vec<ValueType>,
    pub results: Vec<ValueType>,
}

pub struct Limits {
    pub min: u32,
    pub max: Option<u32>,
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

#[derive(Wasmbin)]
pub struct MemoryType {
    pub limits: Limits,
}

#[derive(Wasmbin)]
pub enum ElemType {
    FuncRef = 0x70,
}

#[derive(Wasmbin)]
pub struct TableType {
    pub elem_type: ElemType,
    pub limits: Limits,
}

#[derive(Wasmbin)]
pub struct GlobalType {
    pub value_type: ValueType,
    pub mutable: bool,
}
