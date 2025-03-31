pub trait VecExtension<T> {
    fn extract_first_element_with<F>(&mut self, f: F) -> Option<T>
    where
        F: Fn(&T) -> bool;

    fn empty_to_none(self) -> Option<Vec<T>>;
}

impl<T> VecExtension<T> for Vec<T> {
    fn extract_first_element_with<F>(&mut self, f: F) -> Option<T>
    where
        F: Fn(&T) -> bool,
    {
        for (index, item) in self.iter().enumerate() {
            if f(item) {
                return Some(self.swap_remove(index));
            }
        }
        None
    }

    fn empty_to_none(self) -> Option<Vec<T>> {
        if self.is_empty() {
            None
        } else {
            Some(self)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_first_element_with_some() {
        let mut vec = vec![1, 2, 3, 4, 5, 6];
        assert_eq!(vec.extract_first_element_with(|v| *v == 4), Some(4));
        assert_eq!(vec, vec![1, 2, 3, 6, 5]);
    }

    #[test]
    fn extract_first_element_with_none() {
        let mut vec = vec![1, 2, 3, 4, 5, 6];
        assert!(vec.extract_first_element_with(|v| *v == 0).is_none());
    }

    #[test]
    fn extract_first_element_with_first() {
        let mut vec = vec![(1, 1), (2, 2), (2, 3), (2, 4), (4, 5), (6, 7)];
        assert_eq!(vec.extract_first_element_with(|v| v.0 == 2), Some((2, 2)));
        assert_eq!(vec, vec![(1, 1), (6, 7), (2, 3), (2, 4), (4, 5)]);
    }

    #[test]
    fn empty_to_none_some() {
        let vec = vec![(1, 1), (2, 2), (2, 3), (2, 4), (4, 5), (6, 7)];
        assert_eq!(vec.clone().empty_to_none(), Some(vec));
    }

    #[test]
    fn empty_to_none_none() {
        let vec: Vec<u64> = Vec::new();
        assert!(vec.empty_to_none().is_none());
    }
}
