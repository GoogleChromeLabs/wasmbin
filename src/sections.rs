use crate::builtins::blob::{Blob, RawBlob};
use crate::indices::{FuncIdx, GlobalIdx, MemIdx, TableIdx, TypeIdx};
use crate::instructions::Expression;
use crate::types::{FuncType, GlobalType, MemType, TableType, ValueType};
use crate::{
    wasmbin_discriminants, DecodeError, Wasmbin, WasmbinCountable, WasmbinDecode,
    WasmbinDecodeWithDiscriminant, WasmbinEncode,
};
use arbitrary::Arbitrary;
use custom_debug::CustomDebug;

#[derive(Wasmbin, CustomDebug, Arbitrary, PartialEq, Eq, Hash, Clone)]
pub struct CustomSection {
    pub name: String,

    #[debug(with = "custom_debug::hexbuf_str")]
    pub data: Vec<u8>,
}

#[wasmbin_discriminants]
#[derive(Wasmbin, Debug, Arbitrary, PartialEq, Eq, Hash, Clone)]
#[repr(u8)]
pub enum ImportDesc {
    Func(TypeIdx) = 0x00,
    Table(TableType) = 0x01,
    Mem(MemType) = 0x02,
    Global(GlobalType) = 0x03,
}

#[derive(Wasmbin, Debug, Arbitrary, PartialEq, Eq, Hash, Clone)]
pub struct ImportPath {
    pub module: String,
    pub name: String,
}

#[derive(Wasmbin, WasmbinCountable, Debug, Arbitrary, PartialEq, Eq, Hash, Clone)]
pub struct Import {
    pub path: ImportPath,
    pub desc: ImportDesc,
}

#[derive(Wasmbin, WasmbinCountable, Debug, Arbitrary, PartialEq, Eq, Hash, Clone)]
pub struct Global {
    pub ty: GlobalType,
    pub init: Expression,
}

#[wasmbin_discriminants]
#[derive(Wasmbin, Debug, Arbitrary, PartialEq, Eq, Hash, Clone)]
#[repr(u8)]
pub enum ExportDesc {
    Func(FuncIdx) = 0x00,
    Table(TableIdx) = 0x01,
    Mem(MemIdx) = 0x02,
    Global(GlobalIdx) = 0x03,
}

#[derive(Wasmbin, WasmbinCountable, Debug, Arbitrary, PartialEq, Eq, Hash, Clone)]
pub struct Export {
    pub name: String,
    pub desc: ExportDesc,
}

#[derive(Wasmbin, WasmbinCountable, Debug, Arbitrary, PartialEq, Eq, Hash, Clone)]
pub struct Element {
    pub table: TableIdx,
    pub offset: Expression,
    pub init: Vec<FuncIdx>,
}

#[derive(Wasmbin, WasmbinCountable, Debug, Arbitrary, PartialEq, Eq, Hash, Clone)]
pub struct Locals {
    pub repeat: u32,
    pub ty: ValueType,
}

#[derive(Wasmbin, WasmbinCountable, Debug, Default, Arbitrary, PartialEq, Eq, Hash, Clone)]
pub struct Func {
    pub locals: Vec<Locals>,
    pub body: Expression,
}

#[derive(Wasmbin, WasmbinCountable, CustomDebug, Arbitrary, PartialEq, Eq, Hash, Clone)]
pub struct Data {
    pub memory: MemIdx,
    pub offset: Expression,
    #[debug(with = "custom_debug::hexbuf_str")]
    pub init: RawBlob,
}

#[wasmbin_discriminants]
#[derive(Wasmbin, Debug, Arbitrary, PartialEq, Eq, Hash, Clone)]
#[repr(u8)]
pub enum Section {
    Custom(Blob<CustomSection>) = 0,
    Type(Blob<Vec<FuncType>>) = 1,
    Import(Blob<Vec<Import>>) = 2,
    Function(Blob<Vec<TypeIdx>>) = 3,
    Table(Blob<Vec<TableType>>) = 4,
    Memory(Blob<Vec<MemType>>) = 5,
    Global(Blob<Vec<Global>>) = 6,
    Export(Blob<Vec<Export>>) = 7,
    Start(Blob<FuncIdx>) = 8,
    Element(Blob<Vec<Element>>) = 9,
    Code(Blob<Vec<Blob<Func>>>) = 10,
    Data(Blob<Vec<Data>>) = 11,
}

impl WasmbinEncode for [Section] {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        for section in self {
            section.encode(w)?;
        }
        Ok(())
    }
}

impl WasmbinDecode for Vec<Section> {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        let mut sections = Vec::new();
        loop {
            match u8::decode(r) {
                Err(DecodeError::Io(err)) if err.kind() == std::io::ErrorKind::UnexpectedEof => {
                    return Ok(sections);
                }
                Err(err) => {
                    return Err(err);
                }
                Ok(discriminant) => {
                    sections.push(Section::decode_with_discriminant(discriminant, r)?);
                }
            }
        }
    }
}
