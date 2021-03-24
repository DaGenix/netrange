/// Types that implements the `Range` trait represents a
/// [CIDR](https://en.wikipedia.org/wiki/Classless_Inter-Domain_Routing)
/// range.
///
/// You generally can just use the built in types that implement
/// `Range` in this library: [`IpRange`](crate::IpRange), [`Ipv4Range`](crate::Ipv4Range),
/// and [`Ipv6Range`](crate::Ipv6Range). However, those types are not available in
/// no_std mode which is a case where it may be necessary to implement this trait by the user.
pub trait Range: Clone + Sized {
    type Address: Clone + Ord;

    /// Return the next larger range or [`None`] if there is
    /// no such valid range.
    ///
    /// # Examples
    ///
    /// 1. `127.0.0.0/25` embiggens to `Some(127.0.0.0/24)`.
    /// 2. `127.0.1.0/24` embiggens to `None`.
    /// 2. `0.0.0.0/0` embiggens to `None`.
    fn embiggen(&self) -> Option<Self>;

    /// Return the host address for the range.
    ///
    /// # Example
    ///
    /// If the range represents `127.0.0.0/24`, this must
    /// return a reference to the address `127.0.0.0`.
    fn host_address(&self) -> &Self::Address;

    /// Return the prefix length of the range.
    ///
    /// # Example
    ///
    /// If the range represents `127.0.0.0/24`, this must
    /// return the value `24`.
    fn prefix_length(&self) -> u8;

    /// Return `true` if this is an ipv6 range and `false` otherwise.
    fn is_ipv6(&self) -> bool;

    /// Return `true` if this range contains the other range and
    /// `false` otherwise.
    ///
    /// # Panic
    ///
    /// This method may panic if `self` and `other` do not return
    /// the same value for [`is_ipv6`](Self::is_ipv6).
    ///
    /// # Examples
    ///
    /// 1. `127.0.0.0/24` contains `127.0.0.128/25`.
    /// 2. `127.0.0.128/25` does not contain `127.0.0.0/24`.
    /// 3. `127.0.0.0/24` contains `127.0.0.0/24`.
    /// 4. `0.0.0.0/0` contains any other ipv4 network.
    /// 5. `127.0.0.1/32` contains only itself.
    fn contains(&self, other: &Self) -> bool;
}
