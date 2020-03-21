use crate::builtins::blob::{Blob, RawBlob};
use crate::indices::{FuncIdx, GlobalIdx, MemIdx, TableIdx, TypeIdx};
use crate::instructions::Expression;
use crate::types::{FuncType, GlobalType, MemType, TableType, ValueType};
use crate::{
    DecodeError, Wasmbin, WasmbinCountable, WasmbinDecode, WasmbinDecodeWithDiscriminant,
    WasmbinEncode,
};
use arbitrary::Arbitrary;
use custom_debug::CustomDebug;

#[derive(Wasmbin, CustomDebug, Arbitrary, PartialEq, Eq)]
pub struct CustomSection {
    pub name: String,

    #[debug(with = "custom_debug::hexbuf_str")]
    pub data: Vec<u8>,
}

#[derive(Wasmbin, Debug, Arbitrary, PartialEq, Eq)]
pub enum ImportDesc {
    #[wasmbin(discriminant = 0x00)]
    Func(TypeIdx),

    #[wasmbin(discriminant = 0x01)]
    Table(TableType),

    #[wasmbin(discriminant = 0x02)]
    Mem(MemType),

    #[wasmbin(discriminant = 0x03)]
    Global(GlobalType),
}

#[derive(Wasmbin, Debug, Arbitrary, PartialEq, Eq)]
pub struct ImportPath {
    pub module: String,
    pub name: String,
}

#[derive(Wasmbin, WasmbinCountable, Debug, Arbitrary, PartialEq, Eq)]
pub struct Import {
    pub path: ImportPath,
    pub desc: ImportDesc,
}

#[derive(Wasmbin, WasmbinCountable, Debug, Arbitrary, PartialEq, Eq)]
pub struct Global {
    pub ty: GlobalType,
    pub init: Expression,
}

#[derive(Wasmbin, Debug, Arbitrary, PartialEq, Eq)]
pub enum ExportDesc {
    #[wasmbin(discriminant = 0x00)]
    Func(FuncIdx),

    #[wasmbin(discriminant = 0x01)]
    Table(TableIdx),

    #[wasmbin(discriminant = 0x02)]
    Mem(MemIdx),

    #[wasmbin(discriminant = 0x03)]
    Global(GlobalIdx),
}

#[derive(Wasmbin, WasmbinCountable, Debug, Arbitrary, PartialEq, Eq)]
pub struct Export {
    pub name: String,
    pub desc: ExportDesc,
}

#[derive(Wasmbin, WasmbinCountable, Debug, Arbitrary, PartialEq, Eq)]
pub struct Element {
    pub table: TableIdx,
    pub offset: Expression,
    pub init: Vec<FuncIdx>,
}

#[derive(Wasmbin, WasmbinCountable, Debug, Arbitrary, PartialEq, Eq)]
pub struct Locals {
    pub repeat: u32,
    pub ty: ValueType,
}

#[derive(Wasmbin, WasmbinCountable, Debug, Default, Arbitrary, PartialEq, Eq)]
pub struct Func {
    pub locals: Vec<Locals>,
    pub body: Expression,
}

#[derive(Wasmbin, WasmbinCountable, CustomDebug, Arbitrary, PartialEq, Eq)]
pub struct Data {
    pub memory: MemIdx,
    pub offset: Expression,
    #[debug(with = "custom_debug::hexbuf_str")]
    pub init: RawBlob,
}

#[derive(Wasmbin, Debug, Arbitrary, PartialEq, Eq)]
pub enum Section {
    #[wasmbin(discriminant = 0)]
    Custom(Blob<CustomSection>),

    #[wasmbin(discriminant = 1)]
    Type(Blob<Vec<FuncType>>),

    #[wasmbin(discriminant = 2)]
    Import(Blob<Vec<Import>>),

    #[wasmbin(discriminant = 3)]
    Function(Blob<Vec<TypeIdx>>),

    #[wasmbin(discriminant = 4)]
    Table(Blob<Vec<TableType>>),

    #[wasmbin(discriminant = 5)]
    Memory(Blob<Vec<MemType>>),

    #[wasmbin(discriminant = 6)]
    Global(Blob<Vec<Global>>),

    #[wasmbin(discriminant = 7)]
    Export(Blob<Vec<Export>>),

    #[wasmbin(discriminant = 8)]
    Start(Blob<FuncIdx>),

    #[wasmbin(discriminant = 9)]
    Element(Blob<Vec<Element>>),

    #[wasmbin(discriminant = 10)]
    Code(Blob<Vec<Blob<Func>>>),

    #[wasmbin(discriminant = 11)]
    Data(Blob<Vec<Data>>),
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
