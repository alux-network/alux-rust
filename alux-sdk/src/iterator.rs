use alux_ext::ext;

/// Iterator utility functions.
#[ext(name = IteratorExt)]
pub impl<This> This
where
    This: Iterator,
{
    /// Omits the item at one zero-based position.
    fn skip_nth(self, position: usize) -> impl Iterator<Item = This::Item> {
        self.enumerate().filter_map(move |(index, item)| (index != position).then_some(item))
    }

    /// Includes items through the first item satisfying `predicate`, then stops.
    fn stop_if<Predicate>(self, mut predicate: Predicate) -> impl Iterator<Item = This::Item>
    where
        Predicate: FnMut(&This::Item) -> bool,
    {
        self.scan(false, move |stopped, item| {
            if *stopped {
                None
            } else {
                *stopped = predicate(&item);
                Some(item)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::IteratorExt;

    #[test]
    fn test_skip_nth() {
        assert_eq!((0..4).skip_nth(0).collect::<Vec<_>>(), vec![1, 2, 3]);
        assert_eq!((0..4).skip_nth(2).collect::<Vec<_>>(), vec![0, 1, 3]);
        assert_eq!((0..4).skip_nth(3).collect::<Vec<_>>(), vec![0, 1, 2]);
    }

    #[test]
    fn test_stop_if() {
        assert_eq!((0..10).stop_if(|value| *value > 2).collect::<Vec<_>>(), vec![0, 1, 2, 3]);
    }
}
