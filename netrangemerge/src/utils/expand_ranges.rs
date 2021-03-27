use cidr::{Cidr as _, Inet as _};
use libnetrangemerge::{IpRange, Range, RangeInterest};
use std::cmp;

pub fn expand_ranges(
    ranges: &mut Vec<RangeInterest<IpRange>>,
    min_ipv4_network_size: Option<u8>,
    min_ipv6_network_size: Option<u8>,
) {
    for net in ranges.iter_mut() {
        let min_size = if net.range().is_ipv6() && min_ipv6_network_size.is_some() {
            Some(min_ipv6_network_size.unwrap())
        } else if !net.range().is_ipv6() && min_ipv4_network_size.is_some() {
            Some(min_ipv4_network_size.unwrap())
        } else {
            None
        };
        if let Some(min_size) = min_size {
            let new_prefix_length = cmp::min(net.range().prefix_length(), min_size);
            let inet = cidr::IpInet::new(*net.range().host_address(), new_prefix_length).unwrap();
            *net.range_mut() =
                IpRange::new(inet.network().first_address(), new_prefix_length).unwrap();
        }
    }
}
