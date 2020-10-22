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

use crate::builtins::Lazy;
use crate::builtins::WasmbinCountable;
use crate::builtins::{Blob, RawBlob};
use crate::indices::{FuncId, GlobalId, LocalId, MemId, TableId, TypeId};
use crate::instructions::Expression;
use crate::io::{Decode, DecodeError, DecodeWithDiscriminant, Encode, Wasmbin};
#[cfg(feature = "bulk-memory-operations")]
use crate::types::RefType;
use crate::types::{FuncType, GlobalType, MemType, TableType, ValueType};
use crate::visit::Visit;
use crate::wasmbin_discriminants;
use arbitrary::Arbitrary;
use custom_debug::CustomDebug;
use std::convert::TryFrom;

#[derive(Wasmbin, Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
pub struct ModuleNameSubSection {
    pub name: String,
}

#[derive(Wasmbin, Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
pub struct NameAssoc<I, V> {
    pub index: I,
    pub value: V,
}

impl<I, V> WasmbinCountable for NameAssoc<I, V> {}

#[derive(Wasmbin, Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
pub struct NameMap<I, V> {
    pub items: Vec<NameAssoc<I, V>>,
}

#[wasmbin_discriminants]
#[derive(Wasmbin, Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
#[repr(u8)]
pub enum NameSubSection {
    Module(Blob<String>) = 0,
    Func(Blob<NameMap<FuncId, String>>) = 1,
    Local(Blob<NameMap<FuncId, NameMap<LocalId, String>>>) = 2,
}

impl Encode for [NameSubSection] {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        for sub in self {
            sub.encode(w)?;
        }
        Ok(())
    }
}

impl Decode for Vec<NameSubSection> {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        let mut sub = Vec::new();
        while let Some(disc) = Option::decode(r)? {
            sub.push(NameSubSection::decode_with_discriminant(disc, r)?);
        }
        Ok(sub)
    }
}

#[derive(Wasmbin, WasmbinCountable, Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
pub struct ProducerField {
    pub name: String,
    pub values: Vec<ProducerVersionedName>,
}

#[derive(Wasmbin, WasmbinCountable, Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
pub struct ProducerVersionedName {
    name: String,
    version: String,
}

#[derive(Wasmbin, CustomDebug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
pub struct RawCustomSection {
    pub name: String,

    #[debug(with = "custom_debug::hexbuf_str")]
    pub data: Vec<u8>,
}

macro_rules! define_custom_sections {
    ($($name:ident($ty:ty) = $disc:literal,)*) => {
        #[derive(Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
        pub enum CustomSection {
            $($name(Lazy<$ty>),)*
            Other(RawCustomSection),
        }

        impl Encode for CustomSection {
            fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
                match self {
                    $(CustomSection::$name(data) => {
                        $disc.encode(w)?;
                        data.encode(w)
                    })*
                    CustomSection::Other(raw) => raw.encode(w)
                }
            }
        }

        impl Decode for CustomSection {
            fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
                let raw = RawCustomSection::decode(r)?;
                Ok(match raw.name.as_str() {
                    $($disc => CustomSection::$name(Lazy::from_raw(raw.data)),)*
                    _ => CustomSection::Other(raw)
                })
            }
        }
    };
}

define_custom_sections! {
    Name(Vec<NameSubSection>) = "name",
    Producers(Vec<ProducerField>) = "producers",
}

#[wasmbin_discriminants]
#[derive(Wasmbin, Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
#[repr(u8)]
pub enum ImportDesc {
    Func(TypeId) = 0x00,
    Table(TableType) = 0x01,
    Mem(MemType) = 0x02,
    Global(GlobalType) = 0x03,
}

#[derive(Wasmbin, Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
pub struct ImportPath {
    pub module: String,
    pub name: String,
}

#[derive(Wasmbin, WasmbinCountable, Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
pub struct Import {
    pub path: ImportPath,
    pub desc: ImportDesc,
}

#[derive(Wasmbin, WasmbinCountable, Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
pub struct Global {
    pub ty: GlobalType,
    pub init: Expression,
}

#[wasmbin_discriminants]
#[derive(Wasmbin, Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
#[repr(u8)]
pub enum ExportDesc {
    Func(FuncId) = 0x00,
    Table(TableId) = 0x01,
    Mem(MemId) = 0x02,
    Global(GlobalId) = 0x03,
}

#[derive(Wasmbin, WasmbinCountable, Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
pub struct Export {
    pub name: String,
    pub desc: ExportDesc,
}

#[cfg(feature = "bulk-memory-operations")]
#[derive(Wasmbin, Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
#[repr(u8)]
pub enum ElemKind {
    FuncRef = 0x00,
}

#[wasmbin_discriminants]
#[derive(Wasmbin, WasmbinCountable, Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
#[repr(u8)]
pub enum Element {
    ActiveWithFuncs {
        offset: Expression,
        funcs: Vec<FuncId>,
    } = 0x00,
    #[cfg(feature = "bulk-memory-operations")]
    PassiveWithFuncs { kind: ElemKind, funcs: Vec<FuncId> } = 0x01,
    #[cfg(feature = "bulk-memory-operations")]
    ActiveWithTableAndFuncs {
        table: TableId,
        offset: Expression,
        kind: ElemKind,
        funcs: Vec<FuncId>,
    } = 0x02,
    #[cfg(feature = "reference-types")]
    DeclarativeWithFuncs { kind: ElemKind, funcs: Vec<FuncId> } = 0x03,
    #[cfg(feature = "bulk-memory-operations")]
    ActiveWithExprs {
        offset: Expression,
        exprs: Vec<Expression>,
    } = 0x04,
    #[cfg(feature = "bulk-memory-operations")]
    PassiveWithExprs { ty: RefType, exprs: Vec<Expression> } = 0x05,
    #[cfg(feature = "bulk-memory-operations")]
    ActiveWithTableAndExprs {
        table: TableId,
        offset: Expression,
        ty: RefType,
        exprs: Vec<Expression>,
    } = 0x06,
    #[cfg(feature = "reference-types")]
    DeclarativeWithExprs { ty: RefType, exprs: Vec<Expression> } = 0x07,
}

#[derive(Wasmbin, WasmbinCountable, Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
pub struct Locals {
    pub repeat: u32,
    pub ty: ValueType,
}

#[derive(
    Wasmbin, WasmbinCountable, Debug, Default, Arbitrary, PartialEq, Eq, Hash, Clone, Visit,
)]
pub struct FuncBody {
    pub locals: Vec<Locals>,
    pub expr: Expression,
}

#[wasmbin_discriminants]
#[derive(Wasmbin, Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
#[repr(u8)]
pub enum DataInit {
    Active {
        offset: Expression,
    } = 0x00,
    #[cfg(feature = "bulk-memory-operations")]
    Passive = 0x01,
    #[cfg(feature = "bulk-memory-operations")]
    ActiveWithMemory {
        memory: MemId,
        offset: Expression,
    } = 0x02,
}

#[derive(Wasmbin, WasmbinCountable, CustomDebug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
pub struct Data {
    pub init: DataInit,
    #[debug(with = "custom_debug::hexbuf_str")]
    pub blob: RawBlob,
}

pub trait Payload: Encode + Decode + Into<Section> {
    const KIND: Kind;

    fn try_from_ref(section: &Section) -> Option<&Blob<Self>>;
    fn try_from_mut(section: &mut Section) -> Option<&mut Blob<Self>>;
    fn try_from(section: Section) -> Result<Blob<Self>, Section>;
}

pub trait StdPayload: Payload {}

macro_rules! define_sections {
    ($($(# $attr:tt)? $name:ident($ty:ty) = $disc:literal,)*) => {
        pub mod payload {
            $($(# $attr)? pub type $name = $ty;)*
        }

        #[wasmbin_discriminants]
        #[derive(Wasmbin, Debug, Arbitrary, PartialEq, Eq, Hash, Clone, Visit)]
        #[repr(u8)]
        pub enum Section {
            $($(# $attr)? $name(Blob<payload::$name>) = $disc,)*
        }

        #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
        #[repr(u8)]
        pub enum Kind {
            $($(# $attr)? $name = $disc,)*
        }

        impl TryFrom<u8> for Kind {
            type Error = u8;

            fn try_from(discriminant: u8) -> Result<Kind, u8> {
                Ok(match discriminant {
                    $($(# $attr)? $disc => Kind::$name,)*
                    _ => return Err(discriminant),
                })
            }
        }

        impl Ord for Kind {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                // Some new sections might have larger discriminants,
                // but be ordered logically between those will smaller
                // discriminants.
                //
                // To compare their Kinds in a defined order, we need an
                // intermediate enum without discriminants.
                #[derive(PartialEq, Eq, PartialOrd, Ord)]
                #[repr(u8)]
                enum OrderedRepr {
                    $($(# $attr)? $name,)*
                }

                impl From<Kind> for OrderedRepr {
                    fn from(kind: Kind) -> Self {
                        match kind {
                            $($(# $attr)? Kind::$name => Self::$name,)*
                        }
                    }
                }

                OrderedRepr::from(*self).cmp(&OrderedRepr::from(*other))
            }
        }

        impl PartialOrd for Kind {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        $($(# $attr)? const _: () = {
            impl From<Blob<payload::$name>> for Section {
                fn from(value: Blob<payload::$name>) -> Self {
                    Section::$name(value)
                }
            }

            impl From<payload::$name> for Section {
                fn from(value: payload::$name) -> Self {
                    Section::$name(Blob::from(value))
                }
            }

            impl Payload for payload::$name {
                const KIND: Kind = Kind::$name;

                fn try_from_ref(section: &Section) -> Option<&Blob<Self>> {
                    match section {
                        Section::$name(res) => Some(res),
                        _ => None,
                    }
                }

                fn try_from_mut(section: &mut Section) -> Option<&mut Blob<Self>> {
                    match section {
                        Section::$name(res) => Some(res),
                        _ => None,
                    }
                }

                fn try_from(section: Section) -> Result<Blob<Self>, Section> {
                    match section {
                        Section::$name(res) => Ok(res),
                        _ => Err(section),
                    }
                }
            }
        };)*

        impl Section {
            pub fn kind(&self) -> Kind {
                match self {
                    $($(# $attr)? Section::$name(_) => Kind::$name,)*
                }
            }

            pub fn try_as<T: Payload>(&self) -> Option<&Blob<T>> {
                T::try_from_ref(self)
            }

            pub fn try_as_mut<T: Payload>(&mut self) -> Option<&mut Blob<T>> {
                T::try_from_mut(self)
            }
        }

        define_sections!(@std $($(# $attr)? $name)*);
    };

    (@std $ignore_custom:ident $($(# $attr:tt)? $name:ident)*) => {
        $($(# $attr)? impl StdPayload for payload::$name {})*
    };
}

define_sections! {
    Custom(super::CustomSection) = 0,
    Type(Vec<super::FuncType>) = 1,
    Import(Vec<super::Import>) = 2,
    Function(Vec<super::TypeId>) = 3,
    Table(Vec<super::TableType>) = 4,
    Memory(Vec<super::MemType>) = 5,
    Global(Vec<super::Global>) = 6,
    Export(Vec<super::Export>) = 7,
    Start(super::FuncId) = 8,
    Element(Vec<super::Element>) = 9,
    #[cfg(feature = "bulk-memory-operations")]
    DataCount(u32) = 12,
    Code(Vec<super::Blob<super::FuncBody>>) = 10,
    Data(Vec<super::Data>) = 11,
}

impl Encode for [Section] {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        for section in self {
            section.encode(w)?;
        }
        Ok(())
    }
}

impl Decode for Vec<Section> {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        let mut sections = Vec::new();
        let mut last_kind = Kind::Custom;
        while let Some(disc) = Option::decode(r)? {
            match Kind::try_from(disc) {
                Ok(Kind::Custom) | Err(_) => {}
                Ok(kind) => {
                    if kind <= last_kind {
                        return Err(DecodeError::SectionOutOfOrder {
                            prev: last_kind,
                            current: kind,
                        });
                    }
                    last_kind = kind;
                }
            }
            sections.push(Section::decode_with_discriminant(disc, r)?);
        }
        Ok(sections)
    }
}
