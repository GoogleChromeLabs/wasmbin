use super::Wasmbin;

macro_rules! newtype_idx {
    ($name:ident) => {
        #[derive(Debug, PartialEq, Eq, Clone, Copy, Wasmbin)]
        pub struct $name(u32);
    };
}

newtype_idx!(TypeIdx);
newtype_idx!(FuncIdx);
newtype_idx!(TableIdx);
newtype_idx!(MemIdx);
newtype_idx!(GlobalIdx);
newtype_idx!(LocalIdx);
newtype_idx!(LabelIdx);
