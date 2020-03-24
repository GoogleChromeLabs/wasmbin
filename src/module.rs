use crate::builtins::Blob;
use crate::io::{DecodeError, Wasmbin, WasmbinDecode, WasmbinEncode};
use crate::sections::{Section, StdPayload};
use crate::visit::WasmbinVisit;
use arbitrary::Arbitrary;
use std::cmp::Ordering;

const MAGIC_AND_VERSION: [u8; 8] = [b'\0', b'a', b's', b'm', 0x01, 0x00, 0x00, 0x00];

#[derive(Debug, Default, Arbitrary, PartialEq, Eq, Hash, Clone, WasmbinVisit)]
pub struct MagicAndVersion;

impl WasmbinEncode for MagicAndVersion {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        w.write_all(&MAGIC_AND_VERSION)
    }
}

impl WasmbinDecode for MagicAndVersion {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        let mut magic_and_version = [0; 8];
        r.read_exact(&mut magic_and_version)?;
        if magic_and_version != MAGIC_AND_VERSION {
            return Err(DecodeError::InvalidMagic);
        }
        Ok(MagicAndVersion)
    }
}

#[derive(Wasmbin, Debug, Default, Arbitrary, PartialEq, Eq, Hash, Clone, WasmbinVisit)]
pub struct Module {
    #[doc(hidden)]
    pub magic_and_version: MagicAndVersion,
    pub sections: Vec<Section>,
}

impl Module {
    pub fn find_std_section<T: StdPayload>(&self) -> Option<&Blob<T>> {
        self.sections.iter().find_map(|section| section.try_as())
    }

    pub fn find_std_section_mut<T: StdPayload>(&mut self) -> Option<&mut Blob<T>> {
        self.sections
            .iter_mut()
            .find_map(|section| section.try_as_mut())
    }

    pub fn find_or_insert_std_section<T: StdPayload>(
        &mut self,
        insert_callback: impl FnOnce() -> T,
    ) -> &mut Blob<T> {
        let mut index = self.sections.len();
        let mut insert = true;
        for (i, section) in self.sections.iter_mut().enumerate() {
            match section.kind().cmp(&T::KIND) {
                Ordering::Less => continue,
                Ordering::Equal => {
                    // We can't just `return` here due to a bug in rustc:
                    // https://github.com/rust-lang/rust/issues/70255
                    insert = false;
                }
                Ordering::Greater => {}
            }
            index = i;
            break;
        }
        if insert {
            self.sections.insert(index, insert_callback().into());
        }
        unsafe { self.sections.get_unchecked_mut(index) }
            .try_as_mut()
            .unwrap_or_else(|| unsafe { std::hint::unreachable_unchecked() })
    }
}
