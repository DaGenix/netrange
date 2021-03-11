#![cfg_attr(not(feature = "std"), no_std)]

mod merge;
mod network;
mod network_interest;
#[cfg(feature = "std")]
mod std_network;

pub use merge::merge_networks_slice;
pub use network::Network;
pub use network_interest::NetworkInterest;
#[cfg(feature = "std")]
pub use std_network::{
    InvalidHostAddressError, InvalidNetworkError, InvalidNetworkLengthError, IpNetwork,
    Ipv4Network, Ipv6Network, NetworkParseError, UnparseableNetworkError,
};

pub fn merge_networks<N: Network>(networks: &mut Vec<NetworkInterest<N>>) {
    let len = merge_networks_slice(networks);
    networks.truncate(len)
}
