use crate::Range;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum State<R>
where
    R: Range,
{
    Normal { range: R, selected: bool },
    Dummy,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct RangeInterest<R>
where
    R: Range,
{
    state: State<R>,
}

/// A `RangeInterest` represents a network range (some type
/// that implements [`Range](crate::Range)) and a boolean flag
/// that indicates if the range is selected - where
/// selected is an application defined concept.
///
/// # Panics
///
/// [`merge_ranges_slice`](crate::merge_ranges_slice) merges a slice
/// of `NetworkInterest` values in place and returns the size of the
/// result set. `NetworkInterest` values _after_ this returned size
/// are left in an invalid state and any attempt to call a method
/// on them may panic.
impl<R: Range> RangeInterest<R> {
    /// Create a new `RangeInterest` with the given `range` value and
    /// `selected` flag.
    pub fn new(range: R, selected: bool) -> RangeInterest<R> {
        RangeInterest {
            state: State::Normal { range, selected },
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

    /// Return the value of the `selected` flag.
    pub fn is_selected(&self) -> bool {
        match &self.state {
            State::Normal { selected, .. } => *selected,
            State::Dummy => panic!("RangeInterest is invalid"),
        }
    }

    /// Set the value of the `selected` flag.
    pub fn set_selected(&mut self, selected: bool) {
        match &mut self.state {
            State::Normal { selected: int, .. } => *int = selected,
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
