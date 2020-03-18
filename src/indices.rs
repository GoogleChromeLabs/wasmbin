use crate::{Wasmbin, WasmbinCountable};

macro_rules! newtype_idx {
    ($name:ident) => {
        #[derive(PartialEq, Eq, Clone, Copy, Wasmbin, WasmbinCountable)]
        #[repr(transparent)]
        pub struct $name {
            pub index: u32,
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(
                    f,
                    "{}#{}",
                    &stringify!($name)[..stringify!($name).len() - "Idx".len()],
                    self.index
                )
            }
        }
    };
}

newtype_idx!(TypeIdx);
newtype_idx!(FuncIdx);
newtype_idx!(TableIdx);
newtype_idx!(MemIdx);
newtype_idx!(GlobalIdx);
newtype_idx!(LocalIdx);
newtype_idx!(LabelIdx);
