pub trait Network: Sized {
    type Address;

    fn embiggen(&self) -> Option<Self>;

    fn host_address(&self) -> &Self::Address;

    fn network_length(&self) -> u8;

    fn is_ipv6(&self) -> bool;

    fn contains(&self, other: &Self) -> bool;
}
