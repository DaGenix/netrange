mod merge;
mod network;

pub use network::{
    InvalidHostAddressError, InvalidNetworkError, InvalidNetworkLengthError, Network,
    NetworkParseError, UnparseableNetworkError,
};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum NetworkInterestState {
    Interesting,
    NotInteresting,
    Dummy,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct NetworkInterest {
    network: Network,
    state: NetworkInterestState,
}

impl NetworkInterest {
    pub fn new(network: Network, interesting: bool) -> NetworkInterest {
        let state = if interesting {
            NetworkInterestState::Interesting
        } else {
            NetworkInterestState::NotInteresting
        };
        NetworkInterest { network, state }
    }

    pub fn network(&self) -> Network {
        if self.is_dummy() {
            panic!("NetworkInterest is invalid");
        }
        self.network
    }

    pub fn is_interesting(&self) -> bool {
        match self.state {
            NetworkInterestState::Interesting => true,
            NetworkInterestState::NotInteresting => false,
            NetworkInterestState::Dummy => panic!("NetworkInterest is invalid"),
        }
    }

    pub(crate) fn set_dummy(&mut self) {
        self.state = NetworkInterestState::Dummy;
    }

    pub(crate) fn is_dummy(&self) -> bool {
        if let NetworkInterestState::Dummy = self.state {
            true
        } else {
            false
        }
    }
}
