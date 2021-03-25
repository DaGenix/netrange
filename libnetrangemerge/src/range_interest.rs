use crate::Range;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum State<R>
where
    R: Range,
{
    Normal { range: R, interesting: bool },
    Dummy,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct RangeInterest<R>
where
    R: Range,
{
    state: State<R>,
}

/// A `RangeInterest` represents a network range object (some type
/// that implements [`Range](crate::Range) and a boolean flag
/// that indicates if the range is interesting to the user - where
/// interesting is a user defined concept.
///
/// # Panics
///
/// After a slice of `NetworkInterest` value are merged with
/// [`merge_ranges_slice`](crate::merge_ranges_slice) we will end up
/// with a smaller number of valid result ranges than we had in the
/// input. In this case, the extra `RangeInterest` values are left
/// in an invalid state and an attempt to call any method on them
/// will `panic!()`
impl<R: Range> RangeInterest<R> {
    /// Create a new `RangeInterest` with the given `range` value and
    /// `interesting` flag.
    pub fn new(range: R, interesting: bool) -> RangeInterest<R> {
        RangeInterest {
            state: State::Normal { range, interesting },
        }
    }

    /// Return a shared reference to the contained `range` value.
    pub fn range(&self) -> &R {
        match &self.state {
            State::Normal { range, .. } => range,
            State::Dummy => panic!("RangeInterest is invalid"),
        }
    }

    /// Return a mutable reference to the contained `range` value.
    pub fn range_mut(&mut self) -> &mut R {
        match &mut self.state {
            State::Normal { range, .. } => range,
            State::Dummy => panic!("RangeInterest is invalid"),
        }
    }

    /// Return the value of the `interesting` flag.
    pub fn is_interesting(&self) -> bool {
        match &self.state {
            State::Normal { interesting, .. } => *interesting,
            State::Dummy => panic!("RangeInterest is invalid"),
        }
    }

    /// Return the value of the `interesting` flag.
    pub fn set_interesting(&mut self, intersting: bool) {
        match &mut self.state {
            State::Normal {
                interesting: int, ..
            } => *int = intersting,
            State::Dummy => panic!("RangeInterest is invalid"),
        }
    }

    /// Unwrap the `NetworkInterest` and return the contained
    /// `range` value.
    pub fn unwrap(self) -> R {
        match self.state {
            State::Normal { range, .. } => range,
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
