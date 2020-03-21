use arbitrary::Arbitrary;
use once_cell::unsync::OnceCell;

#[cfg(debug_assertions)]
fn unreachable<T>() -> T {
    unreachable!()
}

#[cfg(not(debug_assertions))]
fn unreachable<T>() -> T {
    unsafe { std::hint::unreachable_unchecked() }
}

#[cfg(feature = "nightly")]
pub type NoError = !;

#[cfg(not(feature = "nightly"))]
pub enum NoError {}

pub trait LazyTransform {
    type Input;
    type Output;
    type Error;

    fn lazy_transform(input: &Self::Input) -> Result<Self::Output, Self::Error>;
}

pub struct LazyMut<L: LazyTransform> {
    input: Option<L::Input>,
    output: OnceCell<L::Output>,
}

impl<L: LazyTransform> std::fmt::Debug for LazyMut<L>
where
    L::Input: std::fmt::Debug,
    L::Output: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("LazyMut")
            .field("input", &self.input_opt())
            .field("output", &self.output_opt())
            .finish()
    }
}

impl<L: 'static + LazyTransform> Arbitrary for LazyMut<L>
where
    L::Output: Arbitrary,
{
    fn arbitrary(u: &mut arbitrary::Unstructured) -> arbitrary::Result<Self> {
        L::Output::arbitrary(u).map(LazyMut::new_from_output)
    }

    fn arbitrary_take_rest(u: arbitrary::Unstructured) -> arbitrary::Result<Self> {
        L::Output::arbitrary_take_rest(u).map(LazyMut::new_from_output)
    }

    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        L::Output::size_hint(depth)
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        match self.output.get() {
            Some(output) => Box::new(output.shrink().map(LazyMut::new_from_output)),
            None => Box::new(std::iter::empty()),
        }
    }
}

impl<L: LazyTransform> Default for LazyMut<L>
where
    L::Input: Default,
    L::Output: Default,
{
    fn default() -> Self {
        Self::new_with_output_unchecked(Default::default(), Default::default())
    }
}

impl<L: LazyTransform> LazyMut<L> {
    pub fn new(input: L::Input) -> Self {
        LazyMut {
            input: Some(input),
            output: OnceCell::new(),
        }
    }

    pub fn new_from_output(output: L::Output) -> Self {
        LazyMut {
            input: None,
            output: OnceCell::from(output),
        }
    }

    pub fn new_with_output_unchecked(input: L::Input, output: L::Output) -> Self {
        LazyMut {
            input: Some(input),
            output: OnceCell::from(output),
        }
    }

    pub fn input_opt(&self) -> Option<&L::Input> {
        self.input.as_ref()
    }

    pub fn input_res(&self) -> Result<&L::Input, &L::Output> {
        self.input_opt()
            .ok_or_else(|| self.output_opt().unwrap_or_else(unreachable))
    }

    pub fn output_opt(&self) -> Option<&L::Output> {
        self.output.get()
    }

    pub fn output_opt_mut(&mut self) -> Option<&mut L::Output> {
        self.input = None;
        self.output.get_mut()
    }

    pub fn set_output(&mut self, output: L::Output) {
        self.input = None;
        self.output = OnceCell::from(output);
    }

    pub fn try_output(&self) -> Result<&L::Output, L::Error> {
        let input = &self.input;
        self.output
            .get_or_try_init(|| L::lazy_transform(input.as_ref().unwrap_or_else(unreachable)))
    }

    pub fn try_output_mut(&mut self) -> Result<&mut L::Output, L::Error> {
        self.try_output()?;
        self.output_opt_mut().ok_or_else(unreachable)
    }

    pub fn try_into_output(self) -> Result<L::Output, L::Error> {
        self.try_output()?;
        self.output.into_inner().ok_or_else(unreachable)
    }
}

impl<L: LazyTransform<Error = NoError>> LazyMut<L> {
    pub fn output(&self) -> &L::Output {
        self.try_output().unwrap_or_else(|err| match err {})
    }

    pub fn output_mut(&mut self) -> &mut L::Output {
        self.try_output_mut().unwrap_or_else(|err| match err {})
    }

    pub fn into_output(self) -> L::Output {
        self.try_into_output().unwrap_or_else(|err| match err {})
    }
}

impl<L: LazyTransform<Error = NoError>> std::ops::Deref for LazyMut<L> {
    type Target = L::Output;

    fn deref(&self) -> &L::Output {
        self.output()
    }
}

impl<L: LazyTransform<Error = NoError>> std::ops::DerefMut for LazyMut<L> {
    fn deref_mut(&mut self) -> &mut L::Output {
        self.output_mut()
    }
}

impl<L: LazyTransform> PartialEq for LazyMut<L>
where
    L::Input: PartialEq,
    L::Output: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        // Fast path: both LazyMuts are in the "input" state and inputs are equal.
        // Assume that outputs will be equal too, and don't invoke transforms.
        // If inputs are not equal, it doesn't yet mean that outputs are not equal
        // too, so carry on with full output comparison.
        if let (Some(a), Some(b)) = (self.input_opt(), other.input_opt()) {
            if a == b {
                return true;
            }
        }
        // Slower path: inputs are not equal or at least one of the LazyMuts
        // is in the "output" state.
        match (self.try_output(), other.try_output()) {
            (Ok(a), Ok(b)) => a == b,
            // If at least one of the transforms errored out, treat containers
            // as unequal.
            _ => false,
        }
    }
}

impl<L: LazyTransform> Eq for LazyMut<L>
where
    L::Input: Eq,
    L::Output: Eq,
{
}

impl<L: LazyTransform> std::hash::Hash for LazyMut<L>
where
    L::Output: std::hash::Hash,
{
    fn hash<H: std::hash::Hasher>(&self, h: &mut H) {
        // Unlike in case with equality, we must take a hash of the output
        // here so that we can find some value later when it's evaluated.
        self.try_output().ok().hash(h)
    }
}

impl<L: LazyTransform> Clone for LazyMut<L>
where
    L::Input: Clone,
    L::Output: Clone,
{
    fn clone(&self) -> Self {
        LazyMut {
            input: self.input.clone(),
            output: self.output.clone(),
        }
    }
}
