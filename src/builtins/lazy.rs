use crate::builtins::WasmbinCountable;
use crate::io::{DecodeError, WasmbinDecode, WasmbinEncode};
use crate::visit::{VisitError, WasmbinVisit};
use arbitrary::Arbitrary;
use custom_debug::CustomDebug;
use once_cell::unsync::OnceCell;
use std::hash::Hash;

#[derive(CustomDebug, Clone)]
enum LazyStatus<T> {
    FromInput {
        #[debug(with = "custom_debug::hexbuf_str")]
        raw: Vec<u8>,
        parsed: OnceCell<T>,
    },
    Output {
        value: T,
    },
}

#[derive(Clone)]
pub struct Lazy<T> {
    status: LazyStatus<T>,
}

impl<T> Lazy<T> {
    pub fn from_raw(raw: Vec<u8>) -> Self {
        Lazy {
            status: LazyStatus::FromInput {
                raw,
                parsed: OnceCell::new(),
            },
        }
    }

    pub fn try_as_raw(&self) -> Result<&[u8], &T> {
        match &self.status {
            LazyStatus::FromInput { raw, .. } => Ok(raw),
            LazyStatus::Output { value } => Err(value),
        }
    }
}

impl<T> From<T> for Lazy<T> {
    fn from(value: T) -> Self {
        Lazy {
            status: LazyStatus::Output { value },
        }
    }
}

impl<T: Default> Default for Lazy<T> {
    fn default() -> Self {
        T::default().into()
    }
}

impl<T: WasmbinEncode> WasmbinEncode for Lazy<T> {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        match &self.status {
            LazyStatus::FromInput { raw, .. } => raw.encode(w),
            LazyStatus::Output { value } => value.encode(w),
        }
    }
}

impl<T: WasmbinDecode> WasmbinDecode for Lazy<T> {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        Vec::decode(r).map(Self::from_raw)
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Lazy<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.status.fmt(f)
    }
}

fn decode_raw<T: WasmbinDecode>(mut raw: &[u8]) -> Result<T, DecodeError> {
    let value = T::decode(&mut raw)?;
    if !raw.is_empty() {
        return Err(DecodeError::UnrecognizedData);
    }
    Ok(value)
}

impl<T: WasmbinDecode> Lazy<T> {
    pub fn try_contents(&self) -> Result<&T, DecodeError> {
        match &self.status {
            LazyStatus::FromInput { raw, parsed } => parsed.get_or_try_init(|| decode_raw(raw)),
            LazyStatus::Output { value } => Ok(value),
        }
    }

    pub fn try_contents_mut(&mut self) -> Result<&mut T, DecodeError> {
        if let LazyStatus::FromInput { raw, parsed } = &mut self.status {
            // We can't trust input and output to match once we obtained a mutable reference,
            // so get the value and change the status to just Output.
            let parsed = std::mem::replace(parsed, OnceCell::new());
            self.status = LazyStatus::Output {
                value: match parsed.into_inner() {
                    Some(value) => value,
                    None => decode_raw(raw)?,
                },
            };
        }
        if let LazyStatus::Output { value } = &mut self.status {
            return Ok(value);
        }
        unsafe { std::hint::unreachable_unchecked() }
    }

    pub fn try_into_contents(self) -> Result<T, DecodeError> {
        match self.status {
            LazyStatus::FromInput { raw, parsed } => match parsed.into_inner() {
                Some(value) => Ok(value),
                None => decode_raw(&raw),
            },
            LazyStatus::Output { value } => Ok(value),
        }
    }
}

impl<T: WasmbinDecode + PartialEq> PartialEq for Lazy<T> {
    fn eq(&self, other: &Self) -> bool {
        if let (LazyStatus::FromInput { raw: raw1, .. }, LazyStatus::FromInput { raw: raw2, .. }) =
            (&self.status, &other.status)
        {
            if raw1 == raw2 {
                return true;
            }
        }
        if let (Ok(value1), Ok(value2)) = (self.try_contents(), other.try_contents()) {
            return value1 == value2;
        }
        return false;
    }
}

impl<T: WasmbinDecode + Eq> Eq for Lazy<T> {}

impl<T: WasmbinDecode + Hash> Hash for Lazy<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.try_contents().ok().hash(state);
    }
}

impl<T: Arbitrary> Arbitrary for Lazy<T> {
    fn arbitrary(u: &mut arbitrary::Unstructured) -> arbitrary::Result<Self> {
        T::arbitrary(u).map(Self::from)
    }

    fn arbitrary_take_rest(u: arbitrary::Unstructured) -> arbitrary::Result<Self> {
        T::arbitrary_take_rest(u).map(Self::from)
    }

    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        T::size_hint(depth)
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        match &self.status {
            LazyStatus::Output { value } => Box::new(value.shrink().map(Self::from)),
            _ => unreachable!(),
        }
    }
}

impl<T: WasmbinCountable> WasmbinCountable for Lazy<T> {}

impl<T: WasmbinDecode + WasmbinVisit> WasmbinVisit for Lazy<T> {
    fn visit_children<'a, VisitT: 'static, E, F: FnMut(&'a VisitT) -> Result<(), E>>(
        &'a self,
        f: &mut F,
    ) -> Result<(), VisitError<E>> {
        match self.try_contents() {
            Ok(contents) => contents.visit_child(f),
            Err(err) => Err(VisitError::LazyDecode(err)),
        }
    }

    fn visit_children_mut<VisitT: 'static, E, F: FnMut(&mut VisitT) -> Result<(), E>>(
        &mut self,
        f: &mut F,
    ) -> Result<(), VisitError<E>> {
        match self.try_contents_mut() {
            Ok(contents) => contents.visit_child_mut(f),
            Err(err) => Err(VisitError::LazyDecode(err)),
        }
    }
}
