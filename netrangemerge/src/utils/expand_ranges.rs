use cidr::{Cidr as _, Inet as _};
use libnetrangemerge::{IpRange, Range, RangeInterest};
use std::cmp;

pub fn expand_ranges<'a, I>(
    mut ranges: I,
    min_ipv4_network_size: Option<u8>,
    min_ipv6_network_size: Option<u8>,
)
    where I: Iterator<Item=&'a mut IpRange>
{
    for range in ranges {
        let min_size = if range.is_ipv6() && min_ipv6_network_size.is_some() {
            Some(min_ipv6_network_size.unwrap())
        } else if !range.is_ipv6() && min_ipv4_network_size.is_some() {
            Some(min_ipv4_network_size.unwrap())
        } else {
            None
        };
        if let Some(min_size) = min_size {
            let new_prefix_length = cmp::min(range.prefix_length(), min_size);
            let inet = cidr::IpInet::new(*range.host_address(), new_prefix_length).unwrap();
            *range =
                IpRange::new(inet.network().first_address(), new_prefix_length).unwrap();
        }
    }
}
