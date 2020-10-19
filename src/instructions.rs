// Copyright 2020 Google Inc. All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::builtins::FloatConst;
#[cfg(feature = "bulk-memory-operations")]
use crate::indices::{DataId, ElemId};
use crate::indices::{FuncId, GlobalId, LabelId, LocalId, MemId, TableId, TypeId};
use crate::io::{Decode, DecodeError, DecodeWithDiscriminant, Encode, Wasmbin};
use crate::types::BlockType;
#[cfg(feature = "bulk-memory-operations")]
use crate::types::RefType;
#[cfg(feature = "reference-types")]
use crate::types::ValueType;
use crate::visit::Visit;
use crate::wasmbin_discriminants;
use arbitrary::Arbitrary;

const OP_CODE_BLOCK_START: u8 = 0x02;
const OP_CODE_LOOP_START: u8 = 0x03;
const OP_CODE_IF_START: u8 = 0x04;
const OP_CODE_END: u8 = 0x0B;

impl Encode for [Instruction] {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        for instr in self {
            instr.encode(w)?;
        }
        OP_CODE_END.encode(w)
    }
}

impl Decode for Vec<Instruction> {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        let mut res = Vec::new();
        let mut depth: usize = 0;
        loop {
            let op_code = u8::decode(r)?;
            match op_code {
                OP_CODE_BLOCK_START | OP_CODE_LOOP_START | OP_CODE_IF_START => {
                    depth += 1;
                }
                OP_CODE_END => match depth.checked_sub(1) {
                    Some(new_depth) => depth = new_depth,
                    None => break,
                },
                _ => {}
            }
            res.push(Instruction::decode_with_discriminant(op_code, r)?);
        }
        Ok(res)
    }
}

pub type Expression = Vec<Instruction>;

impl crate::builtins::WasmbinCountable for Expression {}

#[derive(Wasmbin, Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
pub struct MemArg {
    pub align: u32,
    pub offset: u32,
}

#[derive(Wasmbin, Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
pub struct CallIndirect {
    ty: TypeId,
    table: TableId,
}

#[wasmbin_discriminants]
#[derive(Wasmbin, Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
#[repr(u8)]
pub enum Instruction {
    Unreachable = 0x00,
    Nop = 0x01,
    BlockStart(BlockType) = OP_CODE_BLOCK_START,
    LoopStart(BlockType) = OP_CODE_LOOP_START,
    IfStart(BlockType) = OP_CODE_IF_START,
    IfElse = 0x05,
    End = OP_CODE_END,
    Br(LabelId) = 0x0C,
    BrIf(LabelId) = 0x0D,
    BrTable {
        branches: Vec<LabelId>,
        otherwise: LabelId,
    } = 0x0E,
    Return = 0x0F,
    Call(FuncId) = 0x10,
    CallIndirect(CallIndirect) = 0x11,
    #[cfg(feature = "tail-call")]
    ReturnCall(FuncId) = 0x12,
    #[cfg(feature = "tail-call")]
    ReturnCallIndirect(CallIndirect) = 0x13,
    Drop = 0x1A,
    Select = 0x1B,
    #[cfg(feature = "reference-types")]
    SelectWithTypes(Vec<ValueType>) = 0x1C,
    LocalGet(LocalId) = 0x20,
    LocalSet(LocalId) = 0x21,
    LocalTee(LocalId) = 0x22,
    GlobalGet(GlobalId) = 0x23,
    GlobalSet(GlobalId) = 0x24,
    #[cfg(feature = "reference-types")]
    TableGet(TableId) = 0x25,
    #[cfg(feature = "reference-types")]
    TableSet(TableId) = 0x26,
    I32Load(MemArg) = 0x28,
    I64Load(MemArg) = 0x29,
    F32Load(MemArg) = 0x2A,
    F64Load(MemArg) = 0x2B,
    I32Load8S(MemArg) = 0x2C,
    I32Load8U(MemArg) = 0x2D,
    I32Load16S(MemArg) = 0x2E,
    I32Load16U(MemArg) = 0x2F,
    I64Load8S(MemArg) = 0x30,
    I64Load8U(MemArg) = 0x31,
    I64Load16S(MemArg) = 0x32,
    I64Load16U(MemArg) = 0x33,
    I64Load32S(MemArg) = 0x34,
    I64Load32U(MemArg) = 0x35,
    I32Store(MemArg) = 0x36,
    I64Store(MemArg) = 0x37,
    F32Store(MemArg) = 0x38,
    F64Store(MemArg) = 0x39,
    I32Store8(MemArg) = 0x3A,
    I32Store16(MemArg) = 0x3B,
    I64Store8(MemArg) = 0x3C,
    I64Store16(MemArg) = 0x3D,
    I64Store32(MemArg) = 0x3E,
    MemorySize(MemId) = 0x3F,
    MemoryGrow(MemId) = 0x40,
    I32Const(i32) = 0x41,
    I64Const(i64) = 0x42,
    F32Const(FloatConst<f32>) = 0x43,
    F64Const(FloatConst<f64>) = 0x44,
    I32Eqz = 0x45,
    I32Eq = 0x46,
    I32Ne = 0x47,
    I32LtS = 0x48,
    I32LtU = 0x49,
    I32GtS = 0x4A,
    I32GtU = 0x4B,
    I32LeS = 0x4C,
    I32LeU = 0x4D,
    I32GeS = 0x4E,
    I32GeU = 0x4F,
    I64Eqz = 0x50,
    I64Eq = 0x51,
    I64Ne = 0x52,
    I64LtS = 0x53,
    I64LtU = 0x54,
    I64GtS = 0x55,
    I64GtU = 0x56,
    I64LeS = 0x57,
    I64LeU = 0x58,
    I64GeS = 0x59,
    I64GeU = 0x5A,
    F32Eq = 0x5B,
    F32Ne = 0x5C,
    F32Lt = 0x5D,
    F32Gt = 0x5E,
    F32Le = 0x5F,
    F32Ge = 0x60,
    F64Eq = 0x61,
    F64Ne = 0x62,
    F64Lt = 0x63,
    F64Gt = 0x64,
    F64Le = 0x65,
    F64Ge = 0x66,
    I32Clz = 0x67,
    I32Ctz = 0x68,
    I32PopCnt = 0x69,
    I32Add = 0x6A,
    I32Sub = 0x6B,
    I32Mul = 0x6C,
    I32DivS = 0x6D,
    I32DivU = 0x6E,
    I32RemS = 0x6F,
    I32RemU = 0x70,
    I32And = 0x71,
    I32Or = 0x72,
    I32Xor = 0x73,
    I32Shl = 0x74,
    I32ShrS = 0x75,
    I32ShrU = 0x76,
    I32RotL = 0x77,
    I32RotR = 0x78,
    I64Clz = 0x79,
    I64Ctz = 0x7A,
    I64PopCnt = 0x7B,
    I64Add = 0x7C,
    I64Sub = 0x7D,
    I64Mul = 0x7E,
    I64DivS = 0x7F,
    I64DivU = 0x80,
    I64RemS = 0x81,
    I64RemU = 0x82,
    I64And = 0x83,
    I64Or = 0x84,
    I64Xor = 0x85,
    I64Shl = 0x86,
    I64ShrS = 0x87,
    I64ShrU = 0x88,
    I64RotL = 0x89,
    I64RotR = 0x8A,
    F32Abs = 0x8B,
    F32Neg = 0x8C,
    F32Ceil = 0x8D,
    F32Floor = 0x8E,
    F32Trunc = 0x8F,
    F32Nearest = 0x90,
    F32Sqrt = 0x91,
    F32Add = 0x92,
    F32Sub = 0x93,
    F32Mul = 0x94,
    F32Div = 0x95,
    F32Min = 0x96,
    F32Max = 0x97,
    F32CopySign = 0x98,
    F64Abs = 0x99,
    F64Neg = 0x9A,
    F64Ceil = 0x9B,
    F64Floor = 0x9C,
    F64Trunc = 0x9D,
    F64Nearest = 0x9E,
    F64Sqrt = 0x9F,
    F64Add = 0xA0,
    F64Sub = 0xA1,
    F64Mul = 0xA2,
    F64Div = 0xA3,
    F64Min = 0xA4,
    F64Max = 0xA5,
    F64CopySign = 0xA6,
    I32WrapI64 = 0xA7,
    I32TruncF32S = 0xA8,
    I32TruncF332U = 0xA9,
    I32TruncF64S = 0xAA,
    I32TruncF64U = 0xAB,
    I64ExtendI32S = 0xAC,
    I64ExtendI32U = 0xAD,
    I64TruncF32S = 0xAE,
    I64TruncF32U = 0xAF,
    I64TruncF64S = 0xB0,
    I64TruncF64U = 0xB1,
    F32ConvertI32S = 0xB2,
    F32ConvertI32U = 0xB3,
    F32ConvertI64S = 0xB4,
    F32ConvertI64U = 0xB5,
    F32DemoteF64 = 0xB6,
    F64ConvertI32S = 0xB7,
    F64ConvertI32U = 0xB8,
    F64ConvertI64S = 0xB9,
    F64ConvertI64U = 0xBA,
    F64PromoteF32 = 0xBB,
    I32ReinterpretF32 = 0xBC,
    I64ReinterpretF64 = 0xBD,
    F32ReinterpretI32 = 0xBE,
    F64ReinterpretI64 = 0xBF,
    I32Extend8S = 0xC0,
    I32Extend16S = 0xC1,
    I64Extend8S = 0xC2,
    I64Extend16S = 0xC3,
    I64Extend32S = 0xC4,
    #[cfg(feature = "bulk-memory-operations")]
    RefNull(RefType) = 0xD0,
    #[cfg(feature = "reference-types")]
    RefIsNull = 0xD1,
    #[cfg(feature = "bulk-memory-operations")]
    RefFunc(FuncId) = 0xD2,
    Misc(Misc) = 0xFC,
    #[cfg(feature = "simd")]
    SIMD(SIMD) = 0xFD,
}

#[cfg_attr(feature = "bulk-memory-operations", wasmbin_discriminants)]
#[derive(Wasmbin, Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
#[repr(u8)]
pub enum Misc {
    I32TruncSatF32S = 0x00,
    I32TruncSatF32U = 0x01,
    I32TruncSatF64S = 0x02,
    I32TruncSatF64U = 0x03,
    I64TruncSatF32S = 0x04,
    I64TruncSatF32U = 0x05,
    I64TruncSatF64S = 0x06,
    I64TruncSatF64U = 0x07,
    #[cfg(feature = "bulk-memory-operations")]
    MemoryInit {
        data: DataId,
        mem: MemId,
    } = 0x08,
    #[cfg(feature = "bulk-memory-operations")]
    DataDrop(DataId) = 0x09,
    #[cfg(feature = "bulk-memory-operations")]
    MemoryCopy {
        dest: MemId,
        src: MemId,
    } = 0x0A,
    #[cfg(feature = "bulk-memory-operations")]
    MemoryFill(MemId) = 0x0B,
    #[cfg(feature = "bulk-memory-operations")]
    TableInit {
        elem: ElemId,
        table: TableId,
    } = 0x0C,
    #[cfg(feature = "bulk-memory-operations")]
    ElemDrop(ElemId) = 0x0D,
    #[cfg(feature = "bulk-memory-operations")]
    TableCopy {
        dest: MemId,
        src: MemId,
    } = 0x0E,
    #[cfg(feature = "reference-types")]
    TableGrow(TableId) = 0x0F,
    #[cfg(feature = "reference-types")]
    TableSize(TableId) = 0x10,
    #[cfg(feature = "reference-types")]
    TableFill(TableId) = 0x11,
}

#[cfg(feature = "simd")]
#[allow(clippy::wildcard_imports)]
pub mod simd {
    use super::*;

    macro_rules! def_lane_idx {
        ($name:ident, $num:literal) => {
            #[derive(Debug, PartialEq, Eq, Hash, Clone, Visit)]
            #[repr(transparent)]
            pub struct $name(u8);

            impl Encode for $name {
                fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
                    self.0.encode(w)
                }
            }

            impl Decode for $name {
                fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
                    let value = u8::decode(r)?;
                    if value >= $num {
                        return Err(DecodeError::UnsupportedDiscriminant {
                            ty: stringify!($name),
                            discriminant: value.into(),
                        });
                    }
                    Ok(Self(value))
                }
            }

            impl Arbitrary for $name {
                #[allow(clippy::range_minus_one)]
                fn arbitrary(u: &mut arbitrary::Unstructured) -> arbitrary::Result<Self> {
                    u.int_in_range(0..=($num - 1)).map(Self)
                }
            }
        };
    }

    def_lane_idx!(LaneIdx2, 2);
    def_lane_idx!(LaneIdx4, 4);
    def_lane_idx!(LaneIdx8, 8);
    def_lane_idx!(LaneIdx16, 16);
    def_lane_idx!(LaneIdx32, 32);

    impl Encode for [LaneIdx32; 16] {
        fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
            unsafe { &*(self as *const [LaneIdx32; 16] as *const [u8; 16]) }.encode(w)
        }
    }

    impl Decode for [LaneIdx32; 16] {
        fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
            let bytes = <[u8; 16]>::decode(r)?;
            for &b in &bytes {
                if b >= 32 {
                    return Err(DecodeError::UnsupportedDiscriminant {
                        ty: "LaneIdx32",
                        discriminant: b.into(),
                    });
                }
            }
            Ok(unsafe { std::mem::transmute::<[u8; 16], [LaneIdx32; 16]>(bytes) })
        }
    }

    impl_visit_for_iter!([u8; 16]);
    impl_visit_for_iter!([LaneIdx32; 16]);

    #[wasmbin_discriminants]
    #[derive(Wasmbin, Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
    #[repr(u32)]
    pub enum SIMD {
        V128Load(MemArg) = 0x00,
        I16x8Load8x8S(MemArg) = 0x01,
        I16x8Load8x8U(MemArg) = 0x02,
        I32x4Load16x4S(MemArg) = 0x03,
        I32x4Load16x4U(MemArg) = 0x04,
        I64x2Load32x2S(MemArg) = 0x05,
        I64x2Load32x2U(MemArg) = 0x06,
        V8x16LoadSplat(MemArg) = 0x07,
        V16x8LoadSplat(MemArg) = 0x08,
        V32x4LoadSplat(MemArg) = 0x09,
        V64x2LoadSplat(MemArg) = 0x0A,
        V128Store(MemArg) = 0x0B,
        V128Const([u8; 16]) = 0x0C,
        V8x16Shuffle([LaneIdx32; 16]) = 0x0D,
        V8x16Swizzle = 0x0E,
        I8x16Splat = 0x0F,
        I16x8Splat = 0x10,
        I32x4Splat = 0x11,
        I64x2Splat = 0x12,
        F32x4Splat = 0x13,
        F64x2Splat = 0x14,
        I8x16ExtractLaneS(LaneIdx16) = 0x15,
        I8x16ExtractLaneU(LaneIdx16) = 0x16,
        I8x16ReplaceLane(LaneIdx16) = 0x17,
        I16x8ExtractLaneS(LaneIdx8) = 0x18,
        I16x8ExtractLaneU(LaneIdx8) = 0x19,
        I16x8ReplaceLane(LaneIdx8) = 0x1A,
        I32x4ExtractLane(LaneIdx4) = 0x1B,
        I32x4ReplaceLane(LaneIdx4) = 0x1C,
        I64x2ExtractLane(LaneIdx2) = 0x1D,
        I64x2ReplaceLane(LaneIdx2) = 0x1E,
        F32x4ExtractLane(LaneIdx4) = 0x1F,
        F32x4ReplaceLane(LaneIdx4) = 0x20,
        F64x2ExtractLane(LaneIdx2) = 0x21,
        F64x2ReplaceLane(LaneIdx2) = 0x22,
        I8x16Eq = 0x23,
        I8x16Ne = 0x24,
        I8x16LtS = 0x25,
        I8x16LtU = 0x26,
        I8x16GtS = 0x27,
        I8x16GtU = 0x28,
        I8x16LeS = 0x29,
        I8x16LeU = 0x2A,
        I8x16GeS = 0x2B,
        I8x16GeU = 0x2C,
        I16x8Eq = 0x2D,
        I16x8Ne = 0x2E,
        I16x8LtS = 0x2F,
        I16x8LtU = 0x30,
        I16x8GtS = 0x31,
        I16x8GtU = 0x32,
        I16x8LeS = 0x33,
        I16x8LeU = 0x34,
        I16x8GeS = 0x35,
        I16x8GeU = 0x36,
        I32x4Eq = 0x37,
        I32x4Ne = 0x38,
        I32x4LtS = 0x39,
        I32x4LtU = 0x3A,
        I32x4GtS = 0x3B,
        I32x4GtU = 0x3C,
        I32x4LeS = 0x3D,
        I32x4LeU = 0x3E,
        I32x4GeS = 0x3F,
        I32x4GeU = 0x40,
        F32x4Eq = 0x41,
        F32x4Ne = 0x42,
        F32x4Lt = 0x43,
        F32x4Gt = 0x44,
        F32x4Le = 0x45,
        F32x4Ge = 0x46,
        F64x2Eq = 0x47,
        F64x2Ne = 0x48,
        F64x2Lt = 0x49,
        F64x2Gt = 0x4A,
        F64x2Le = 0x4B,
        F64x2Ge = 0x4C,
        V128Not = 0x4D,
        V128And = 0x4E,
        V128Andnot = 0x4F,
        V128Or = 0x50,
        V128Xor = 0x51,
        V128Bitselect = 0x52,
        I8x16Abs = 0x60,
        I8x16Neg = 0x61,
        I8x16AnyTrue = 0x62,
        I8x16AllTrue = 0x63,
        I8x16Bitmask = 0x64,
        I8x16NarrowI16x8S = 0x65,
        I8x16NarrowI16x8U = 0x66,
        I8x16Shl = 0x6B,
        I8x16ShrS = 0x6C,
        I8x16ShrU = 0x6D,
        I8x16Add = 0x6E,
        I8x16AddSaturateS = 0x6F,
        I8x16AddSaturateU = 0x70,
        I8x16Sub = 0x71,
        I8x16SubSaturateS = 0x72,
        I8x16SubSaturateU = 0x73,
        I8x16MinS = 0x76,
        I8x16MinU = 0x77,
        I8x16MaxS = 0x78,
        I8x16MaxU = 0x79,
        I8x16AvgrU = 0x7B,
        I16x8Abs = 0x80,
        I16x8Neg = 0x81,
        I16x8AnyTrue = 0x82,
        I16x8AllTrue = 0x83,
        I16x8Bitmask = 0x84,
        I16x8NarrowI32x4S = 0x85,
        I16x8NarrowI32x4U = 0x86,
        I16x8WidenLowI8x16S = 0x87,
        I16x8WidenHighI8x16S = 0x88,
        I16x8WidenLowI8x16U = 0x89,
        I16x8WidenHighI8x16U = 0x8A,
        I16x8Shl = 0x8B,
        I16x8ShrS = 0x8C,
        I16x8ShrU = 0x8D,
        I16x8Add = 0x8E,
        I16x8AddSaturateS = 0x8F,
        I16x8AddSaturateU = 0x90,
        I16x8Sub = 0x91,
        I16x8SubSaturateS = 0x92,
        I16x8SubSaturateU = 0x93,
        I16x8Mul = 0x95,
        I16x8MinS = 0x96,
        I16x8MinU = 0x97,
        I16x8MaxS = 0x98,
        I16x8MaxU = 0x99,
        I16x8AvgrU = 0x9B,
        I32x4Abs = 0xA0,
        I32x4Neg = 0xA1,
        I32x4AnyTrue = 0xA2,
        I32x4AllTrue = 0xA3,
        I32x4Bitmask = 0xA4,
        I32x4WidenLowI16x8S = 0xA7,
        I32x4WidenHighI16x8S = 0xA8,
        I32x4WidenLowI16x8U = 0xA9,
        I32x4WidenHighI16x8U = 0xAA,
        I32x4Shl = 0xAB,
        I32x4ShrS = 0xAC,
        I32x4ShrU = 0xAD,
        I32x4Add = 0xAE,
        I32x4Sub = 0xB1,
        I32x4Mul = 0xB5,
        I32x4MinS = 0xB6,
        I32x4MinU = 0xB7,
        I32x4MaxS = 0xB8,
        I32x4MaxU = 0xB9,
        I64x2Neg = 0xC1,
        I64x2Shl = 0xCB,
        I64x2ShrS = 0xCC,
        I64x2ShrU = 0xCD,
        I64x2Add = 0xCE,
        I64x2Sub = 0xD1,
        I64x2Mul = 0xD5,
        F32x4Abs = 0xE0,
        F32x4Neg = 0xE1,
        F32x4Sqrt = 0xE3,
        F32x4Add = 0xE4,
        F32x4Sub = 0xE5,
        F32x4Mul = 0xE6,
        F32x4Div = 0xE7,
        F32x4Min = 0xE8,
        F32x4Max = 0xE9,
        F64x2Abs = 0xEC,
        F64x2Neg = 0xED,
        F64x2Sqrt = 0xEF,
        F64x2Add = 0xF0,
        F64x2Sub = 0xF1,
        F64x2Mul = 0xF2,
        F64x2Div = 0xF3,
        F64x2Min = 0xF4,
        F64x2Max = 0xF5,
        I32x4TruncSatF32x4S = 0xF8,
        I32x4TruncSatF32x4U = 0xF9,
        F32x4ConvertI32x4S = 0xFA,
        F32x4ConvertI32x4U = 0xFB,
    }
}

#[cfg(feature = "simd")]
pub use simd::SIMD;
