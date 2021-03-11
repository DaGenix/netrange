#[cfg(feature = "std")]
use libnetrangemerge::{merge_networks, Ipv4Network, NetworkInterest};

#[cfg(feature = "std")]
fn simple_ipv4_data() -> Vec<NetworkInterest<Ipv4Network>> {
    vec![
        NetworkInterest::new("127.0.0.0/31".parse().unwrap(), false),
        NetworkInterest::new("127.0.0.2/31".parse().unwrap(), true),
        NetworkInterest::new("127.0.0.0/30".parse().unwrap(), true),
        NetworkInterest::new("127.0.0.4/30".parse().unwrap(), true),
        NetworkInterest::new("127.0.0.8/31".parse().unwrap(), true),
        NetworkInterest::new("127.0.0.10/31".parse().unwrap(), true),
        NetworkInterest::new("127.0.4.0/23".parse().unwrap(), true),
        NetworkInterest::new("127.0.6.0/23".parse().unwrap(), true),
    ]
}

#[cfg(feature = "std")]
fn main() {
    for _ in 0..1000 {
        merge_networks(&mut simple_ipv4_data());
    }
}

#[cfg(not(feature = "std"))]
fn main() {}
