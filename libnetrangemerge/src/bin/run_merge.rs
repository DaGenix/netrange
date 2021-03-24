#[cfg(feature = "std")]
use libnetrangemerge::{merge_ranges, Ipv4Range, RangeInterest};

#[cfg(feature = "std")]
fn simple_ipv4_data() -> Vec<RangeInterest<Ipv4Range>> {
    vec![
        RangeInterest::new("127.0.0.0/31".parse().unwrap(), false),
        RangeInterest::new("127.0.0.2/31".parse().unwrap(), true),
        RangeInterest::new("127.0.0.0/30".parse().unwrap(), true),
        RangeInterest::new("127.0.0.4/30".parse().unwrap(), true),
        RangeInterest::new("127.0.0.8/31".parse().unwrap(), true),
        RangeInterest::new("127.0.0.10/31".parse().unwrap(), true),
        RangeInterest::new("127.0.4.0/23".parse().unwrap(), true),
        RangeInterest::new("127.0.6.0/23".parse().unwrap(), true),
    ]
}

#[cfg(feature = "std")]
fn main() {
    for _ in 0..1000 {
        merge_ranges(&mut simple_ipv4_data());
    }
}

#[cfg(not(feature = "std"))]
fn main() {}
