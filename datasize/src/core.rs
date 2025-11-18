macro_rules! forward_for_unit_structs {
    ( $($name:path,)* ) => {
        $(
            impl<T> crate::DataSize for $name
            where
                T: crate::DataSize,
            {
                const IS_DYNAMIC: bool = <T as crate::DataSize>::IS_DYNAMIC;

                const STATIC_HEAP_SIZE: usize = <T as crate::DataSize>::STATIC_HEAP_SIZE;

                fn estimate_heap_size(&self) -> usize {
                    self.0.estimate_heap_size()
                }
            }
        )*
    };
}

forward_for_unit_structs!(
    core::panic::AssertUnwindSafe<T>,
    core::cmp::Reverse<T>,
);

crate::non_dynamic_const_heap_size!(
  core::net::Ipv4Addr
  core::net::Ipv6Addr
  core::net::SocketAddrV4
  core::net::SocketAddrV6
  core::net::IpAddr
  core::net::SocketAddr,

  0
);
