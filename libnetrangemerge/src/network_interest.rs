use crate::Network;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum State<N>
where
    N: Network,
{
    Normal { network: N, interesting: bool },
    Dummy,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct NetworkInterest<N>
where
    N: Network,
{
    state: State<N>,
}

impl<N: Network> NetworkInterest<N> {
    pub fn new(network: N, interesting: bool) -> NetworkInterest<N> {
        NetworkInterest {
            state: State::Normal {
                network,
                interesting,
            },
        }
    }

    pub fn network(&self) -> &N {
        match &self.state {
            State::Normal { network, .. } => network,
            State::Dummy => panic!("NetworkInterest is invalid"),
        }
    }

    pub fn is_interesting(&self) -> bool {
        match &self.state {
            State::Normal { interesting, .. } => *interesting,
            State::Dummy => panic!("NetworkInterest is invalid"),
        }
    }

    pub(crate) fn set_dummy(&mut self) {
        self.state = State::Dummy;
    }

    pub(crate) fn is_dummy(&self) -> bool {
        match &self.state {
            State::Normal { .. } => false,
            State::Dummy => true,
        }
    }
}
