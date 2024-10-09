use std::cell::RefCell;

pub struct SerdeIteratorAdapter<I>(RefCell<I>);

impl<I> SerdeIteratorAdapter<I> {
    pub fn new(iterator: I) -> Self {
        Self(RefCell::new(iterator))
    }
}

impl<I> serde::Serialize for SerdeIteratorAdapter<I>
where
    I: Iterator,
    I::Item: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_seq(self.0.borrow_mut().by_ref())
    }
}
