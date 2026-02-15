/// Iterator extension for non-short-circuiting any
pub trait AnyAll: Iterator {
    fn any_all<F>(&mut self, mut f: F) -> bool
    where
        F: FnMut(Self::Item) -> bool,
    {
        let mut result = false;
        for item in self {
            if f(item) {
                result = true;
            }
        }
        result
    }
}

// Implement for all iterators
impl<I: Iterator> AnyAll for I {}
