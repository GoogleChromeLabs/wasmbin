use crate::{Wasmbin, DecodeError, WasmbinEncode, WasmbinDecode};
use crate::types::BlockType;
use crate::indices::{FuncIdx, TableIdx, MemIdx, LabelIdx, LocalIdx, GlobalIdx, TypeIdx};

#[derive(Wasmbin)]
enum SeqInstructionRepr {
    Instruction(Instruction),

    #[wasmbin(discriminant = 0x0B)]
    End,
}

pub struct Instructions(Vec<Instruction>);

impl std::ops::Deref for Instructions {
    type Target = Vec<Instruction>;

    fn deref(&self) -> &Vec<Instruction> {
        &self.0
    }
}

impl std::ops::DerefMut for Instructions {
    fn deref_mut(&mut self) -> &mut Vec<Instruction> {
        &mut self.0
    }
}

impl WasmbinEncode for Instructions {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        Instruction::encode_seq(&self.0, w)?;
        SeqInstructionRepr::End.encode(w)
    }
}

impl WasmbinDecode for Instructions {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        let mut res = Vec::new();
        loop {
            match SeqInstructionRepr::decode(r)? {
                SeqInstructionRepr::Instruction(instr) => res.push(instr),
                SeqInstructionRepr::End => return Ok(Instructions(res)),
            }
        }
    }
}

#[derive(Wasmbin)]
pub struct BlockBody {
    pub return_type: BlockType,
    pub instructions: Instructions,
}

pub struct IfElse {
    pub return_type: BlockType,
    pub then: Vec<Instruction>,
    pub otherwise: Vec<Instruction>,
}

#[derive(Wasmbin)]
enum IfElseInstructionRepr {
    Instruction(SeqInstructionRepr),

    #[wasmbin(discriminant = 0x05)]
    Else,
}

impl WasmbinEncode for IfElse {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        self.return_type.encode(w)?;
        Instruction::encode_seq(&self.then, w)?;
        if !self.otherwise.is_empty() {
            IfElseInstructionRepr::Else.encode(w)?;
            Instruction::encode_seq(&self.otherwise, w)?;
        }
        SeqInstructionRepr::End.encode(w)
    }
}

impl WasmbinDecode for IfElse {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        let mut res = IfElse {
            return_type: BlockType::decode(r)?,
            then: Vec::new(),
            otherwise: Vec::new(),
        };
        loop {
            match IfElseInstructionRepr::decode(r)? {
                IfElseInstructionRepr::Instruction(SeqInstructionRepr::Instruction(instr)) => {
                    res.then.push(instr);
                }
                IfElseInstructionRepr::Instruction(SeqInstructionRepr::End) => {
                    break;
                }
                IfElseInstructionRepr::Else => {
                    res.otherwise = Instructions::decode(r)?.0;
                    break;
                }
            }
        }
        Ok(res)
    }
}

#[derive(Wasmbin)]
pub struct MemArg {
    align: u32,
    offset: u32,
}

#[derive(Wasmbin)]
pub enum Instruction {
    #[wasmbin(discriminant = 0x00)]
    Unreachable,

    #[wasmbin(discriminant = 0x01)]
    Nop,

    #[wasmbin(discriminant = 0x02)]
    Block(BlockBody),

    #[wasmbin(discriminant = 0x03)]
    Loop(BlockBody),

    #[wasmbin(discriminant = 0x04)]
    IfElse(IfElse),

    #[wasmbin(discriminant = 0x0C)]
    Br(LabelIdx),

    #[wasmbin(discriminant = 0x0D)]
    BrIf(LabelIdx),

    #[wasmbin(discriminant = 0x0E)]
    BrTable {
        branches: Vec<LabelIdx>,
        otherwise: LabelIdx,
    },

    #[wasmbin(discriminant = 0x0F)]
    Return,

    #[wasmbin(discriminant = 0x10)]
    Call(FuncIdx),

    #[wasmbin(discriminant = 0x11)]
    CallIndirect { ty: TypeIdx, table: TableIdx },

    #[wasmbin(discriminant = 0x1A)]
    Drop,

    #[wasmbin(discriminant = 0x1B)]
    Select,

    #[wasmbin(discriminant = 0x20)]
    LocalGet(LocalIdx),

    #[wasmbin(discriminant = 0x21)]
    LocalSet(LocalIdx),

    #[wasmbin(discriminant = 0x22)]
    LocalTee(LocalIdx),

    #[wasmbin(discriminant = 0x23)]
    GlobalGet(GlobalIdx),

    #[wasmbin(discriminant = 0x24)]
    GlobalSet(GlobalIdx),

    #[wasmbin(discriminant = 0x28)]
    I32Load(MemArg),

    #[wasmbin(discriminant = 0x29)]
    I64Load(MemArg),

    #[wasmbin(discriminant = 0x2A)]
    F32Load(MemArg),

    #[wasmbin(discriminant = 0x2B)]
    F64Load(MemArg),

    #[wasmbin(discriminant = 0x2C)]
    I32Load8S(MemArg),

    #[wasmbin(discriminant = 0x2D)]
    I32Load8U(MemArg),

    #[wasmbin(discriminant = 0x2E)]
    I32Load16S(MemArg),

    #[wasmbin(discriminant = 0x2F)]
    I32Load16U(MemArg),

    #[wasmbin(discriminant = 0x30)]
    I64Load8S(MemArg),

    #[wasmbin(discriminant = 0x31)]
    I64Load8U(MemArg),

    #[wasmbin(discriminant = 0x32)]
    I64Load16S(MemArg),

    #[wasmbin(discriminant = 0x33)]
    I64Load16U(MemArg),

    #[wasmbin(discriminant = 0x34)]
    I64Load32S(MemArg),

    #[wasmbin(discriminant = 0x35)]
    I64Load32U(MemArg),

    #[wasmbin(discriminant = 0x36)]
    I32Store(MemArg),

    #[wasmbin(discriminant = 0x37)]
    I64Store(MemArg),

    #[wasmbin(discriminant = 0x38)]
    F32Store(MemArg),

    #[wasmbin(discriminant = 0x39)]
    F64Store(MemArg),

    #[wasmbin(discriminant = 0x3A)]
    I32Store8(MemArg),

    #[wasmbin(discriminant = 0x3B)]
    I32Store16(MemArg),

    #[wasmbin(discriminant = 0x3C)]
    I64Store8(MemArg),

    #[wasmbin(discriminant = 0x3D)]
    I64Store16(MemArg),

    #[wasmbin(discriminant = 0x3E)]
    I64Store32(MemArg),

    #[wasmbin(discriminant = 0x3F)]
    MemorySize(MemIdx),

    #[wasmbin(discriminant = 0x40)]
    MemoryGrow(MemIdx),

    #[wasmbin(discriminant = 0x41)]
    I32Const(i32),

    #[wasmbin(discriminant = 0x42)]
    I64Const(i64),

    #[wasmbin(discriminant = 0x43)]
    F32Const(f32),

    #[wasmbin(discriminant = 0x44)]
    F64Const(f64),

    #[wasmbin(discriminant = 0x45)]
    I32Eqz,

    #[wasmbin(discriminant = 0x46)]
    I32Eq,

    #[wasmbin(discriminant = 0x47)]
    I32Ne,

    #[wasmbin(discriminant = 0x48)]
    I32LtS,

    #[wasmbin(discriminant = 0x49)]
    I32LtU,

    #[wasmbin(discriminant = 0x4A)]
    I32GtS,

    #[wasmbin(discriminant = 0x4B)]
    I32GtU,

    #[wasmbin(discriminant = 0x4C)]
    I32LeS,

    #[wasmbin(discriminant = 0x4D)]
    I32LeU,

    #[wasmbin(discriminant = 0x4E)]
    I32GeS,

    #[wasmbin(discriminant = 0x4F)]
    I32GeU,

    #[wasmbin(discriminant = 0x50)]
    I64Eqz,

    #[wasmbin(discriminant = 0x51)]
    I64Eq,

    #[wasmbin(discriminant = 0x52)]
    I64Ne,

    #[wasmbin(discriminant = 0x53)]
    I64LtS,

    #[wasmbin(discriminant = 0x54)]
    I64LtU,

    #[wasmbin(discriminant = 0x55)]
    I64GtS,

    #[wasmbin(discriminant = 0x56)]
    I64GtU,

    #[wasmbin(discriminant = 0x57)]
    I64LeS,

    #[wasmbin(discriminant = 0x58)]
    I64LeU,

    #[wasmbin(discriminant = 0x59)]
    I64GeS,

    #[wasmbin(discriminant = 0x5A)]
    I64GeU,

    #[wasmbin(discriminant = 0x5B)]
    F32Eq,

    #[wasmbin(discriminant = 0x5C)]
    F32Ne,

    #[wasmbin(discriminant = 0x5D)]
    F32Lt,

    #[wasmbin(discriminant = 0x5E)]
    F32Gt,

    #[wasmbin(discriminant = 0x5F)]
    F32Le,

    #[wasmbin(discriminant = 0x60)]
    F32Ge,

    #[wasmbin(discriminant = 0x61)]
    F64Eq,

    #[wasmbin(discriminant = 0x62)]
    F64Ne,

    #[wasmbin(discriminant = 0x63)]
    F64Lt,

    #[wasmbin(discriminant = 0x64)]
    F64Gt,

    #[wasmbin(discriminant = 0x65)]
    F64Le,

    #[wasmbin(discriminant = 0x66)]
    F64Ge,

    #[wasmbin(discriminant = 0x67)]
    I32Clz,

    #[wasmbin(discriminant = 0x68)]
    I32Ctz,

    #[wasmbin(discriminant = 0x69)]
    I32PopCnt,

    #[wasmbin(discriminant = 0x6A)]
    I32Add,

    #[wasmbin(discriminant = 0x6B)]
    I32Sub,

    #[wasmbin(discriminant = 0x6C)]
    I32Mul,

    #[wasmbin(discriminant = 0x6D)]
    I32DivS,

    #[wasmbin(discriminant = 0x6E)]
    I32DivU,

    #[wasmbin(discriminant = 0x6F)]
    I32RemS,

    #[wasmbin(discriminant = 0x70)]
    I32RemU,

    #[wasmbin(discriminant = 0x71)]
    I32And,

    #[wasmbin(discriminant = 0x72)]
    I32Or,

    #[wasmbin(discriminant = 0x73)]
    I32Xor,

    #[wasmbin(discriminant = 0x74)]
    I32Shl,

    #[wasmbin(discriminant = 0x75)]
    I32ShrS,

    #[wasmbin(discriminant = 0x76)]
    I32ShrU,

    #[wasmbin(discriminant = 0x77)]
    I32RotL,

    #[wasmbin(discriminant = 0x78)]
    I32RotR,

    #[wasmbin(discriminant = 0x79)]
    I64Clz,

    #[wasmbin(discriminant = 0x7A)]
    I64Ctz,

    #[wasmbin(discriminant = 0x7B)]
    I64PopCnt,

    #[wasmbin(discriminant = 0x7C)]
    I64Add,

    #[wasmbin(discriminant = 0x7D)]
    I64Sub,

    #[wasmbin(discriminant = 0x7E)]
    I64Mul,

    #[wasmbin(discriminant = 0x7F)]
    I64DivS,

    #[wasmbin(discriminant = 0x80)]
    I64DivU,

    #[wasmbin(discriminant = 0x81)]
    I64RemS,

    #[wasmbin(discriminant = 0x82)]
    I64RemU,

    #[wasmbin(discriminant = 0x83)]
    I64And,

    #[wasmbin(discriminant = 0x84)]
    I64Or,

    #[wasmbin(discriminant = 0x85)]
    I64Xor,

    #[wasmbin(discriminant = 0x86)]
    I64Shl,

    #[wasmbin(discriminant = 0x87)]
    I64ShrS,

    #[wasmbin(discriminant = 0x88)]
    I64ShrU,

    #[wasmbin(discriminant = 0x89)]
    I64RotL,

    #[wasmbin(discriminant = 0x8A)]
    I64RotR,

    #[wasmbin(discriminant = 0x8B)]
    F32Abs,

    #[wasmbin(discriminant = 0x8C)]
    F32Neg,

    #[wasmbin(discriminant = 0x8D)]
    F32Ceil,

    #[wasmbin(discriminant = 0x8E)]
    F32Floor,

    #[wasmbin(discriminant = 0x8F)]
    F32Trunc,

    #[wasmbin(discriminant = 0x90)]
    F32Nearest,

    #[wasmbin(discriminant = 0x91)]
    F32Sqrt,

    #[wasmbin(discriminant = 0x92)]
    F32Add,

    #[wasmbin(discriminant = 0x93)]
    F32Sub,

    #[wasmbin(discriminant = 0x94)]
    F32Mul,

    #[wasmbin(discriminant = 0x95)]
    F32Div,

    #[wasmbin(discriminant = 0x96)]
    F32Min,

    #[wasmbin(discriminant = 0x97)]
    F32Max,

    #[wasmbin(discriminant = 0x98)]
    F32CopySign,

    #[wasmbin(discriminant = 0x99)]
    F64Abs,

    #[wasmbin(discriminant = 0x9A)]
    F64Neg,

    #[wasmbin(discriminant = 0x9B)]
    F64Ceil,

    #[wasmbin(discriminant = 0x9C)]
    F64Floor,

    #[wasmbin(discriminant = 0x9D)]
    F64Trunc,

    #[wasmbin(discriminant = 0x9E)]
    F64Nearest,

    #[wasmbin(discriminant = 0x9F)]
    F64Sqrt,

    #[wasmbin(discriminant = 0xA0)]
    F64Add,

    #[wasmbin(discriminant = 0xA1)]
    F64Sub,

    #[wasmbin(discriminant = 0xA2)]
    F64Mul,

    #[wasmbin(discriminant = 0xA3)]
    F64Div,

    #[wasmbin(discriminant = 0xA4)]
    F64Min,

    #[wasmbin(discriminant = 0xA5)]
    F64Max,

    #[wasmbin(discriminant = 0xA6)]
    F64CopySign,

    #[wasmbin(discriminant = 0xA7)]
    I32WrapI64,

    #[wasmbin(discriminant = 0xA8)]
    I32TruncF32S,

    #[wasmbin(discriminant = 0xA9)]
    I32TruncF332U,

    #[wasmbin(discriminant = 0xAA)]
    I32TruncF64S,

    #[wasmbin(discriminant = 0xAB)]
    I32TruncF64U,

    #[wasmbin(discriminant = 0xAC)]
    I64ExtendI32S,

    #[wasmbin(discriminant = 0xAD)]
    I64ExtendI32U,

    #[wasmbin(discriminant = 0xAE)]
    I64TruncF32S,

    #[wasmbin(discriminant = 0xAF)]
    I64TruncF32U,

    #[wasmbin(discriminant = 0xB0)]
    I64TruncF64S,

    #[wasmbin(discriminant = 0xB1)]
    I64TruncF64U,

    #[wasmbin(discriminant = 0xB2)]
    F32ConvertI32S,

    #[wasmbin(discriminant = 0xB3)]
    F32ConvertI32U,

    #[wasmbin(discriminant = 0xB4)]
    F32ConvertI64S,

    #[wasmbin(discriminant = 0xB5)]
    F32ConvertI64U,

    #[wasmbin(discriminant = 0xB6)]
    F32DemoteF64,

    #[wasmbin(discriminant = 0xB7)]
    F64ConvertI32S,

    #[wasmbin(discriminant = 0xB8)]
    F64ConvertI32U,

    #[wasmbin(discriminant = 0xB9)]
    F64ConvertI64S,

    #[wasmbin(discriminant = 0xBA)]
    F64ConvertI64U,

    #[wasmbin(discriminant = 0xBB)]
    F64PromoteF32,

    #[wasmbin(discriminant = 0xBC)]
    I32ReinterpretF32,

    #[wasmbin(discriminant = 0xBD)]
    I64ReinterpretF64,

    #[wasmbin(discriminant = 0xBE)]
    F32ReinterpretI32,

    #[wasmbin(discriminant = 0xBF)]
    F64ReinterpretI64,
}
