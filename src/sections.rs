use crate::indices::{FuncIdx, GlobalIdx, MemIdx, TableIdx, TypeIdx};
use crate::instructions::Expression;
use crate::types::{FuncType, GlobalType, MemType, TableType, ValueType};
use crate::{DecodeError, Wasmbin, WasmbinDecode, WasmbinEncode};

pub struct CustomSection {
    pub name: String,
    pub data: Vec<u8>,
}

impl WasmbinEncode for CustomSection {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        self.name.encode(w)?;
        w.write_all(&self.data)
    }
}

impl WasmbinDecode for CustomSection {
    fn decode(r: &mut impl std::io::BufRead) -> Result<Self, DecodeError> {
        let name = String::decode(r)?;
        let mut data = Vec::new();
        r.read_to_end(&mut data)?;
        Ok(CustomSection { name, data })
    }
}

#[derive(Wasmbin)]
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

#[derive(Wasmbin)]
pub struct Import {
    pub module: String,
    pub name: String,
    pub desc: ImportDesc,
}

#[derive(Wasmbin)]
pub struct Global {
    pub ty: GlobalType,
    pub init: Expression,
}

#[derive(Wasmbin)]
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

#[derive(Wasmbin)]
pub struct Export {
    pub name: String,
    pub desc: ExportDesc,
}

#[derive(Wasmbin)]
pub struct Element {
    pub table: TableIdx,
    pub offset: Expression,
    pub init: Vec<FuncIdx>,
}

#[derive(Wasmbin)]
pub struct Locals {
    pub repeat: u32,
    pub ty: ValueType,
}

#[derive(Wasmbin)]
pub struct Func {
    pub locals: Vec<Locals>,
    pub body: Expression,
}

pub struct Code {
    pub func: Func,
}

impl WasmbinEncode for Code {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        let mut encoded = Vec::new();
        self.func.encode(&mut encoded)?;
        encoded.encode(w)
    }
}

impl WasmbinDecode for Code {
    fn decode(r: &mut impl std::io::BufRead) -> Result<Self, DecodeError> {
        u32::decode(r)?; // size
        let func = Func::decode(r)?;
        Ok(Code { func })
    }
}

#[derive(Wasmbin)]
pub struct Data {
    pub data: MemIdx,
    pub offset: Expression,
    pub init: Vec<u8>,
}

#[derive(Wasmbin)]
pub enum Section {
    #[wasmbin(discriminant = 0)]
    Custom(CustomSection),

    #[wasmbin(discriminant = 1)]
    Type(Vec<FuncType>),

    #[wasmbin(discriminant = 2)]
    Import(Vec<Import>),

    #[wasmbin(discriminant = 3)]
    Function(Vec<TypeIdx>),

    #[wasmbin(discriminant = 4)]
    Table(Vec<TableType>),

    #[wasmbin(discriminant = 5)]
    Memory(Vec<MemType>),

    #[wasmbin(discriminant = 6)]
    Global(Vec<Global>),

    #[wasmbin(discriminant = 7)]
    Export(Vec<Export>),

    #[wasmbin(discriminant = 8)]
    Start(FuncIdx),

    #[wasmbin(discriminant = 9)]
    Element(Vec<Element>),

    #[wasmbin(discriminant = 10)]
    Code(Vec<Code>),

    #[wasmbin(discriminant = 11)]
    Data(Vec<Data>),
}

pub struct Module {
    pub sections: Vec<Section>,
}

const MAGIC_AND_VERSION: [u8; 8] = [b'\0', b'a', b's', b'm', 0x01, 0x00, 0x00, 0x00];

impl WasmbinEncode for Module {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        w.write_all(&MAGIC_AND_VERSION)?;
        for section in &self.sections {
            section.encode(w)?;
        }
        Ok(())
    }
}

impl WasmbinDecode for Module {
    fn decode(r: &mut impl std::io::BufRead) -> Result<Self, DecodeError> {
        let mut magic_and_version = [0; 8];
        r.read_exact(&mut magic_and_version)?;
        if magic_and_version != MAGIC_AND_VERSION {
            return Err(DecodeError::InvalidMagic);
        }
        let mut sections = Vec::new();
        while !r.fill_buf()?.is_empty() {
            sections.push(Section::decode(r)?);
        }
        Ok(Module { sections })
    }
}
