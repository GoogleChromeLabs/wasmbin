use crate::io::{Decode, DecodeError, Encode};
use crate::visit::{Visit, VisitError};

pub use wasmbin_derive::WasmbinCountable;
pub trait WasmbinCountable {}

impl<T: WasmbinCountable + Encode> Encode for [T] {
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        self.len().encode(w)?;
        for item in self {
            item.encode(w)?;
        }
        Ok(())
    }
}

impl<T> Encode for Vec<T>
where
    [T]: Encode,
{
    fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        self.as_slice().encode(w)
    }
}

impl<T: WasmbinCountable + Decode> Decode for Vec<T> {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
        let count = usize::decode(r)?;
        std::iter::repeat_with(|| T::decode(r))
            .take(count)
            .collect()
    }
}

impl<T: Visit> Visit for Vec<T> {
    fn visit_children<'a, VisitT: 'static, E, F: FnMut(&'a VisitT) -> Result<(), E>>(
        &'a self,
        f: &mut F,
    ) -> Result<(), VisitError<E>> {
        for v in self {
            v.visit_child(f)?;
        }
        Ok(())
    }

    fn visit_children_mut<VisitT: 'static, E, F: FnMut(&mut VisitT) -> Result<(), E>>(
        &mut self,
        f: &mut F,
    ) -> Result<(), VisitError<E>> {
        for v in self {
            v.visit_child_mut(f)?;
        }
        Ok(())
    }
}

impl<T: Visit> Visit for Option<T> {
    fn visit_children<'a, VisitT: 'static, E, F: FnMut(&'a VisitT) -> Result<(), E>>(
        &'a self,
        f: &mut F,
    ) -> Result<(), VisitError<E>> {
        match self {
            Some(v) => v.visit_child(f),
            None => Ok(()),
        }
    }

    fn visit_children_mut<VisitT: 'static, E, F: FnMut(&mut VisitT) -> Result<(), E>>(
        &mut self,
        f: &mut F,
    ) -> Result<(), VisitError<E>> {
        match self {
            Some(v) => v.visit_child_mut(f),
            None => Ok(()),
        }
    }
}
