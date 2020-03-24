use crate::io::{DecodeError, WasmbinDecode, WasmbinEncode};
use crate::visit::{VisitError, WasmbinVisit};

pub use wasmbin_derive::WasmbinCountable;
pub trait WasmbinCountable {}

impl<T: WasmbinCountable + WasmbinEncode> WasmbinEncode for [T] {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        self.len().encode(w)?;
        for item in self {
            item.encode(w)?;
        }
        Ok(())
    }
}

impl<T> WasmbinEncode for Vec<T>
where
    [T]: WasmbinEncode,
{
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        self.as_slice().encode(w)
    }
}

impl<T: WasmbinCountable + WasmbinDecode> WasmbinDecode for Vec<T> {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        let count = usize::decode(r)?;
        std::iter::repeat_with(|| T::decode(r))
            .take(count)
            .collect()
    }
}

impl<T: WasmbinVisit> WasmbinVisit for Vec<T> {
    fn visit_children<'a, VisitT: 'static, E, F: FnMut(&'a VisitT) -> Result<(), E>>(
        &'a self,
        f: &mut F,
    ) -> Result<(), VisitError<E>> {
        for v in self {
            v.visit_child(f)?;
        }
        Ok(())
    }

    fn visit_children_mut<'a, VisitT: 'static, E, F: FnMut(&'a mut VisitT) -> Result<(), E>>(
        &'a mut self,
        f: &mut F,
    ) -> Result<(), VisitError<E>> {
        for v in self {
            v.visit_child_mut(f)?;
        }
        Ok(())
    }
}

impl<T: WasmbinVisit> WasmbinVisit for Option<T> {
    fn visit_children<'a, VisitT: 'static, E, F: FnMut(&'a VisitT) -> Result<(), E>>(
        &'a self,
        f: &mut F,
    ) -> Result<(), VisitError<E>> {
        match self {
            Some(v) => v.visit_child(f),
            None => Ok(()),
        }
    }

    fn visit_children_mut<'a, VisitT: 'static, E, F: FnMut(&'a mut VisitT) -> Result<(), E>>(
        &'a mut self,
        f: &mut F,
    ) -> Result<(), VisitError<E>> {
        match self {
            Some(v) => v.visit_child_mut(f),
            None => Ok(()),
        }
    }
}
