use crate::Range;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum State<N>
where
    N: Range,
{
    Normal { range: N, interesting: bool },
    Dummy,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct RangeInterest<N>
where
    N: Range,
{
    state: State<N>,
}

impl<N: Range> RangeInterest<N> {
    pub fn new(range: N, interesting: bool) -> RangeInterest<N> {
        RangeInterest {
            state: State::Normal {
                range,
                interesting,
            },
        }
    }

    pub fn range(&self) -> &N {
        match &self.state {
            State::Normal { range, .. } => range,
            State::Dummy => panic!("RangeInterest is invalid"),
        }
    }

    pub fn is_interesting(&self) -> bool {
        match &self.state {
            State::Normal { interesting, .. } => *interesting,
            State::Dummy => panic!("RangeInterest is invalid"),
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
