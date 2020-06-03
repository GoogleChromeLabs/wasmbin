use crate::builtins::WasmbinCountable;
use crate::io::Wasmbin;
use crate::visit::Visit;
use arbitrary::Arbitrary;

macro_rules! newtype_id {
    ($name:ident) => {
        #[derive(PartialEq, Eq, Clone, Copy, Wasmbin, WasmbinCountable, Arbitrary, Hash, Visit)]
        #[repr(transparent)]
        pub struct $name {
            pub index: u32,
        }

        impl From<u32> for $name {
            fn from(index: u32) -> Self {
                Self { index }
            }
        }

        impl From<$name> for u32 {
            fn from(id: $name) -> u32 {
                id.index
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(
                    f,
                    "{}#{}",
                    &stringify!($name)[..stringify!($name).len() - "Id".len()],
                    self.index
                )
            }
        }
    };
}

newtype_id!(TypeId);
newtype_id!(FuncId);
newtype_id!(TableId);
newtype_id!(MemId);
newtype_id!(GlobalId);
newtype_id!(LocalId);
newtype_id!(LabelId);

#[cfg(feature = "bulk-memory-operations")]
newtype_id!(ElemId);

#[cfg(feature = "bulk-memory-operations")]
newtype_id!(DataId);
