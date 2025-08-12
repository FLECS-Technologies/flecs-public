pub trait TryAnyExtension: Iterator + Sized {
    fn try_any<F, E>(self, mut f: F) -> Result<bool, E>
    where
        F: FnMut(Self::Item) -> Result<bool, E>,
    {
        for item in self {
            if f(item)? {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

impl<I: Iterator> TryAnyExtension for I {}
