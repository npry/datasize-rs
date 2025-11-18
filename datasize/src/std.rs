use crate::DataSize;

crate::non_dynamic_const_heap_size!(
  std::time::Instant
  std::time::SystemTime,
  0
);

impl DataSize for std::path::PathBuf {
    const IS_DYNAMIC: bool = true;
    const STATIC_HEAP_SIZE: usize = 0;

    #[inline]
    fn estimate_heap_size(&self) -> usize {
        self.capacity()
    }
}

impl DataSize for std::ffi::OsString {
    const IS_DYNAMIC: bool = true;
    const STATIC_HEAP_SIZE: usize = 0;

    #[inline]
    fn estimate_heap_size(&self) -> usize {
        self.capacity()
    }
}

pub(crate) fn estimate_hashbrown_rawtable<T>(capacity: usize) -> usize {
    // https://github.com/rust-lang/hashbrown/blob/v0.12.3/src/raw/mod.rs#L185
    let buckets = if capacity < 8 {
        if capacity < 4 {
            4
        } else {
            8
        }
    } else {
        (capacity * 8 / 7).next_power_of_two()
    };
    // https://github.com/rust-lang/hashbrown/blob/v0.12.3/src/raw/mod.rs#L242
    let size = size_of::<T>();
    // `Group` is u32, u64, or __m128i depending on the CPU architecture.
    // Return a lower bound, ignoring its constant contributions
    // (through ctrl_align and Group::WIDTH, at most 31 bytes).
    let ctrl_offset = size * buckets;
    // Add one byte of "control" metadata per bucket
    ctrl_offset + buckets
}

impl<K, V, S> DataSize for std::collections::HashMap<K, V, S>
where
    K: DataSize,
    V: DataSize,
{
    const IS_DYNAMIC: bool = true;
    const STATIC_HEAP_SIZE: usize = 0;

    #[inline]
    fn estimate_heap_size(&self) -> usize {
        let size = estimate_hashbrown_rawtable::<(K, V)>(self.capacity());

        if K::IS_DYNAMIC || V::IS_DYNAMIC {
            size + self
                .iter()
                .map(|(k, v)| k.estimate_heap_size() + v.estimate_heap_size())
                .sum::<usize>()
        } else {
            size + self.len() * (K::STATIC_HEAP_SIZE + V::STATIC_HEAP_SIZE)
        }
    }
}

impl<T, S> DataSize for std::collections::HashSet<T, S>
where
    T: DataSize,
{
    const IS_DYNAMIC: bool = true;
    const STATIC_HEAP_SIZE: usize = 0;

    #[inline]
    fn estimate_heap_size(&self) -> usize {
        // HashSet<T> is based on HashMap<T, ()>
        let size = estimate_hashbrown_rawtable::<(T, ())>(self.capacity());

        if T::IS_DYNAMIC {
            size + self
                .iter()
                .map(T::estimate_heap_size)
                .sum::<usize>()
        } else {
            size + self.len() * T::STATIC_HEAP_SIZE
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(feature = "detailed")]
    fn test_nested_detailed_struct() {
        use alloc::boxed::Box;
        use std::collections::HashMap;

        use datasize::MemUsageNode;

        use crate as datasize; // Required for the derive macro.
        use crate::{
            DataSize,
            data_size,
        };

        #[derive(DataSize)]
        struct Inner {
            value: Box<u64>,
            dummy: u8,
        }

        #[derive(DataSize)]
        struct Outer {
            a: Box<Inner>,
            b: Inner,
            c: u8,
        }

        let fixture = Outer {
            a: Box::new(Inner {
                value: Box::new(1),
                dummy: 42,
            }),
            b: Inner {
                value: Box::new(2),
                dummy: 42,
            },
            c: 3,
        };

        let detailed = datasize::data_size_detailed(&fixture);

        let mut inner_map = HashMap::new();
        inner_map.insert("value", MemUsageNode::Size(8));
        inner_map.insert("dummy", MemUsageNode::Size(0));

        let mut outer_map = HashMap::new();
        outer_map.insert("a", MemUsageNode::Size(24));
        outer_map.insert("b", MemUsageNode::Detailed(inner_map));
        outer_map.insert("c", MemUsageNode::Size(0));

        let expected = MemUsageNode::Detailed(outer_map);

        assert_eq!(detailed, expected);

        assert_eq!(data_size(&fixture), detailed.total());
    }
}
