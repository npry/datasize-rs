use core::mem::size_of;

use super::DataSize;

impl<A> DataSize for smallvec::SmallVec<A>
where
    A: smallvec::Array,
    A::Item: DataSize,
{
    const IS_DYNAMIC: bool = true;
    const STATIC_HEAP_SIZE: usize = 0;

    #[inline]
    fn estimate_heap_size(&self) -> usize {
        let sz_base = if self.spilled() {
            // At this point, we're very similar to a regular `Vec`.
            self.capacity() * size_of::<A::Item>()
        } else {
            0
        };

        let sz_used = if A::Item::IS_DYNAMIC {
            self.iter()
                .map(DataSize::estimate_heap_size)
                .sum()
        } else {
            self.len() * A::Item::STATIC_HEAP_SIZE
        };

        sz_base + sz_used
    }
}

#[cfg(test)]
mod test {
    use crate::DataSize;

    struct Test;

    impl DataSize for Test {
        const IS_DYNAMIC: bool = false;
        const STATIC_HEAP_SIZE: usize = 1;

        #[inline]
        fn estimate_heap_size(&self) -> usize {
            Self::STATIC_HEAP_SIZE
        }
    }

    #[test]
    fn nonspilled_smallvec_counts_children() {
        let mut v = smallvec::SmallVec::<[Test; 1]>::new();
        assert_eq!(v.estimate_heap_size(), 0);

        v.push(Test);
        assert_eq!(v.estimate_heap_size(), Test::STATIC_HEAP_SIZE);
    }

    #[test]
    fn spilled_smallvec_counts_children() {
        let mut v = smallvec::SmallVec::<[Test; 1]>::new();
        assert_eq!(v.estimate_heap_size(), 0);

        v.push(Test);
        v.push(Test);
        assert_eq!(v.estimate_heap_size(), Test::STATIC_HEAP_SIZE * 2);
    }
}
