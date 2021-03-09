mod merge;
mod network;

use crate::network::DUMMY_NETWORK;
pub use network::{
    InvalidHostAddressError, InvalidNetworkError, InvalidNetworkLengthError, Network,
    NetworkParseError, UnparseableNetworkError,
};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct NetworkInterest {
    network: Network,
    interesting: bool,
}

impl NetworkInterest {
    pub fn new(network: Network, interesting: bool) -> NetworkInterest {
        NetworkInterest {
            network,
            interesting,
        }
    }

    pub fn network(&self) -> Network {
        self.network
    }

    pub fn is_interesting(&self) -> bool {
        self.interesting
    }
}

pub(crate) const DUMMY_NETWORK_INTEREST: NetworkInterest = NetworkInterest {
    network: DUMMY_NETWORK,
    interesting: false,
};

// pub fn merge(ranges: Vec<Range>) -> () {
// }
