use crate::Range;
use cidr::{Cidr, Inet};
use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

/// The prefix_length was invalid (eg: 33 for an ipv4 address)
pub struct InvalidPrefixLengthError {
    prefix_length: u8,
    max_length: u8,
}

impl Debug for InvalidPrefixLengthError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Invalid prefix length: {}. Prefix length must be {} or less",
            self.prefix_length, self.max_length
        )
    }
}

impl Display for InvalidPrefixLengthError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

/// The host_address was not the first address of the described range.
/// For example, attempting to parse `127.0.0.1/24` will result in this
/// error because the first address of the range is actually `127.0.0.0`.
pub struct InvalidHostAddressError {
    host_address: IpAddr,
    prefix_length: u8,
}

impl Debug for InvalidHostAddressError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let ipinet: cidr::IpInet = cidr::Inet::new(self.host_address, self.prefix_length).unwrap();
        write!(
            f,
            "Invalid host address: {} is not the first address in the range: {}",
            ipinet.address(),
            ipinet.network(),
        )
    }
}

impl Display for InvalidHostAddressError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

/// An error indicating why a range instance could not be constructed.
pub enum InvalidRangeError {
    InvalidPrefixLength(InvalidPrefixLengthError),
    InvalidHostAddress(InvalidHostAddressError),
}

impl Debug for InvalidRangeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            InvalidRangeError::InvalidPrefixLength(err) => Debug::fmt(err, f),
            InvalidRangeError::InvalidHostAddress(err) => Debug::fmt(err, f),
        }
    }
}

impl Display for InvalidRangeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl Error for InvalidRangeError {}

/// An error indicating why a range instance could not be parsed.
/// For example, the address part of the string could be invalid (eg: "127.0.0.0.0.0/8").
/// Or, the prefix_length may not be a u8 (eg: "999" or "abc"). Or, the
/// passed in string may be fully invalid (eg: "lasjdskdsl").
pub struct UnparseableRangeError {
    text: String,
}

impl Debug for UnparseableRangeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Either the host or the prefix length portions of {} could not be parsed",
            self.text
        )
    }
}

impl Display for UnparseableRangeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

/// `InvalidRangeError` contains information about why a range
/// instance could not be parsed.
pub enum RangeParseError {
    InvalidPrefixLength(InvalidPrefixLengthError),
    InvalidHostAddress(InvalidHostAddressError),
    UnparseableRange(UnparseableRangeError),
}

impl Debug for RangeParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            RangeParseError::InvalidPrefixLength(err) => Debug::fmt(err, f),
            RangeParseError::InvalidHostAddress(err) => Debug::fmt(err, f),
            RangeParseError::UnparseableRange(err) => Debug::fmt(err, f),
        }
    }
}

impl Display for RangeParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl Error for RangeParseError {}

/// An `IpRange` represents a network range that may be either
/// an ipv4 or an ipv6 range. If you are only working with
/// ipv4 addresses, it may be better to use [`Ipv4Range`] instead
/// as that type is smaller and thus performance somewhat better.
///
/// An `IpRange` may either be constructed using its `new` method
/// or may be parsed from a string using its [`FromStr`] implementation.
///
/// # Example
///
/// ```
/// # use libnetrangemerge::IpRange;
/// let ip_range: IpRange = "127.0.0.0/8".parse().expect("Range was invalid");
/// ```
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct IpRange {
    host_address: IpAddr,
    prefix_length: u8,
}

impl IpRange {
    /// Create a new `IpRange` value.
    pub fn new(host_address: IpAddr, prefix_length: u8) -> Result<IpRange, InvalidRangeError> {
        match host_address {
            IpAddr::V4(_) => {
                if prefix_length > 32 {
                    return Err(InvalidRangeError::InvalidPrefixLength(
                        InvalidPrefixLengthError {
                            prefix_length,
                            max_length: 32,
                        },
                    ));
                }
            }
            IpAddr::V6(_) => {
                if prefix_length > 128 {
                    return Err(InvalidRangeError::InvalidPrefixLength(
                        InvalidPrefixLengthError {
                            prefix_length,
                            max_length: 128,
                        },
                    ));
                }
            }
        }
        let ipinet: cidr::IpInet = cidr::Inet::new(host_address, prefix_length).unwrap();
        if ipinet.first_address() != host_address {
            return Err(InvalidRangeError::InvalidHostAddress(
                InvalidHostAddressError {
                    host_address,
                    prefix_length,
                },
            ));
        }
        Ok(IpRange {
            host_address,
            prefix_length,
        })
    }
}

impl Range for IpRange {
    type Address = IpAddr;

    fn embiggen(&self) -> Option<Self> {
        assert_ne!(self.prefix_length, 0);
        match IpRange::new(self.host_address, self.prefix_length - 1) {
            Ok(n) => Some(n),
            Err(InvalidRangeError::InvalidHostAddress(_)) => return None,
            Err(InvalidRangeError::InvalidPrefixLength(_)) => unreachable!(),
        }
    }

    fn host_address(&self) -> &Self::Address {
        &self.host_address
    }

    fn prefix_length(&self) -> u8 {
        self.prefix_length
    }

    fn is_ipv6(&self) -> bool {
        self.host_address.is_ipv6()
    }

    fn contains(&self, other: &Self) -> bool {
        let c = cidr::IpCidr::new(self.host_address, self.prefix_length).unwrap();
        c.contains(&other.host_address)
    }
}

impl Debug for IpRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.host_address, self.prefix_length)
    }
}

impl Display for IpRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.host_address, self.prefix_length)
    }
}

fn parse_error<T>(s: &str) -> Result<T, RangeParseError> {
    Err(RangeParseError::UnparseableRange(UnparseableRangeError {
        text: s.to_string(),
    }))
}

impl FromStr for IpRange {
    type Err = RangeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let idx_of_slash = if let Some(pos) = s.find("/") {
            pos
        } else {
            return parse_error(s);
        };

        let host_part = &s[0..idx_of_slash];
        let prefix_length_part = &s[idx_of_slash..][1..];

        let host_address = if let Ok(host_address) = host_part.parse() {
            host_address
        } else {
            return parse_error(s);
        };

        let prefix_length = if let Ok(prefix_length) = prefix_length_part.parse() {
            prefix_length
        } else {
            return parse_error(s);
        };

        match IpRange::new(host_address, prefix_length) {
            Ok(range) => Ok(range),
            Err(InvalidRangeError::InvalidPrefixLength(err)) => {
                Err(RangeParseError::InvalidPrefixLength(err))
            }
            Err(InvalidRangeError::InvalidHostAddress(err)) => {
                Err(RangeParseError::InvalidHostAddress(err))
            }
        }
    }
}

/// An `Ipv4Range` represents an ipv4 network range.
///
/// An `Ipv4Range` may either be constructed using its `new` method
/// or may be parsed from a string using its [`FromStr`] implementation.
///
/// # Example
///
/// ```
/// # use libnetrangemerge::Ipv4Range;
/// let ip_range: Ipv4Range = "127.0.0.0/8".parse().expect("Range was invalid");
/// ```
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Ipv4Range {
    host_address: Ipv4Addr,
    prefix_length: u8,
}

impl Ipv4Range {
    /// Create a new `IpRange` value.
    pub fn new(host_address: Ipv4Addr, prefix_length: u8) -> Result<Ipv4Range, InvalidRangeError> {
        if prefix_length > 32 {
            return Err(InvalidRangeError::InvalidPrefixLength(
                InvalidPrefixLengthError {
                    prefix_length,
                    max_length: 32,
                },
            ));
        }
        let ipinet = cidr::Ipv4Inet::new(host_address, prefix_length).unwrap();
        if ipinet.first_address() != host_address {
            return Err(InvalidRangeError::InvalidHostAddress(
                InvalidHostAddressError {
                    host_address: host_address.into(),
                    prefix_length,
                },
            ));
        }
        Ok(Ipv4Range {
            host_address,
            prefix_length,
        })
    }
}

impl Range for Ipv4Range {
    type Address = Ipv4Addr;

    fn embiggen(&self) -> Option<Self> {
        assert_ne!(self.prefix_length, 0);
        match Ipv4Range::new(self.host_address, self.prefix_length - 1) {
            Ok(n) => Some(n),
            Err(InvalidRangeError::InvalidHostAddress(_)) => return None,
            Err(InvalidRangeError::InvalidPrefixLength(_)) => unreachable!(),
        }
    }

    fn host_address(&self) -> &Self::Address {
        &self.host_address
    }

    fn prefix_length(&self) -> u8 {
        self.prefix_length
    }

    fn is_ipv6(&self) -> bool {
        false
    }

    fn contains(&self, other: &Self) -> bool {
        let c = cidr::Ipv4Cidr::new(self.host_address, self.prefix_length).unwrap();
        c.contains(&other.host_address)
    }
}

impl Debug for Ipv4Range {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.host_address, self.prefix_length)
    }
}

impl Display for Ipv4Range {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.host_address, self.prefix_length)
    }
}

impl FromStr for Ipv4Range {
    type Err = RangeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let idx_of_slash = if let Some(pos) = s.find("/") {
            pos
        } else {
            return parse_error(s);
        };

        let host_part = &s[0..idx_of_slash];
        let prefix_length_part = &s[idx_of_slash..][1..];

        let host_address = if let Ok(host_address) = host_part.parse() {
            host_address
        } else {
            return parse_error(s);
        };

        let prefix_length = if let Ok(prefix_length) = prefix_length_part.parse() {
            prefix_length
        } else {
            return parse_error(s);
        };

        match Ipv4Range::new(host_address, prefix_length) {
            Ok(range) => Ok(range),
            Err(InvalidRangeError::InvalidPrefixLength(err)) => {
                Err(RangeParseError::InvalidPrefixLength(err))
            }
            Err(InvalidRangeError::InvalidHostAddress(err)) => {
                Err(RangeParseError::InvalidHostAddress(err))
            }
        }
    }
}

/// An `Ipv6Range` represents an ipv6 network range.
///
/// An `Ipv6Range` may either be constructed using its `new` method
/// or may be parsed from a string using its [`FromStr`] implementation.
///
/// # Example
///
/// ```
/// # use libnetrangemerge::Ipv6Range;
/// let ip_range: Ipv6Range = "2600::/32".parse().expect("Range was invalid");
/// ```
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Ipv6Range {
    host_address: Ipv6Addr,
    prefix_length: u8,
}

impl Ipv6Range {
    /// Create a new `IpRange` value.
    pub fn new(host_address: Ipv6Addr, prefix_length: u8) -> Result<Ipv6Range, InvalidRangeError> {
        if prefix_length > 128 {
            return Err(InvalidRangeError::InvalidPrefixLength(
                InvalidPrefixLengthError {
                    prefix_length,
                    max_length: 128,
                },
            ));
        }
        let ipinet = cidr::Ipv6Inet::new(host_address, prefix_length).unwrap();
        if ipinet.first_address() != host_address {
            return Err(InvalidRangeError::InvalidHostAddress(
                InvalidHostAddressError {
                    host_address: host_address.into(),
                    prefix_length,
                },
            ));
        }
        Ok(Ipv6Range {
            host_address,
            prefix_length,
        })
    }
}

impl Range for Ipv6Range {
    type Address = Ipv6Addr;

    fn embiggen(&self) -> Option<Self> {
        assert_ne!(self.prefix_length, 0);
        match Ipv6Range::new(self.host_address, self.prefix_length - 1) {
            Ok(n) => Some(n),
            Err(InvalidRangeError::InvalidHostAddress(_)) => return None,
            Err(InvalidRangeError::InvalidPrefixLength(_)) => unreachable!(),
        }
    }

    fn host_address(&self) -> &Self::Address {
        &self.host_address
    }

    fn prefix_length(&self) -> u8 {
        self.prefix_length
    }

    fn is_ipv6(&self) -> bool {
        true
    }

    fn contains(&self, other: &Self) -> bool {
        let c = cidr::Ipv6Cidr::new(self.host_address, self.prefix_length).unwrap();
        c.contains(&other.host_address)
    }
}

impl Debug for Ipv6Range {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.host_address, self.prefix_length)
    }
}

impl Display for Ipv6Range {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.host_address, self.prefix_length)
    }
}

impl FromStr for Ipv6Range {
    type Err = RangeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let idx_of_slash = if let Some(pos) = s.find("/") {
            pos
        } else {
            return parse_error(s);
        };

        let host_part = &s[0..idx_of_slash];
        let prefix_length_part = &s[idx_of_slash..][1..];

        let host_address = if let Ok(host_address) = host_part.parse() {
            host_address
        } else {
            return parse_error(s);
        };

        let prefix_length = if let Ok(prefix_length) = prefix_length_part.parse() {
            prefix_length
        } else {
            return parse_error(s);
        };

        match Ipv6Range::new(host_address, prefix_length) {
            Ok(range) => Ok(range),
            Err(InvalidRangeError::InvalidPrefixLength(err)) => {
                Err(RangeParseError::InvalidPrefixLength(err))
            }
            Err(InvalidRangeError::InvalidHostAddress(err)) => {
                Err(RangeParseError::InvalidHostAddress(err))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::std_range::{InvalidRangeError, IpRange, RangeParseError};

    #[test]
    fn test_new_ipv4() {
        IpRange::new("0.0.0.0".parse().unwrap(), 0).unwrap();
        IpRange::new("127.0.0.0".parse().unwrap(), 12).unwrap();
        IpRange::new("127.0.8.0".parse().unwrap(), 21).unwrap();

        match IpRange::new("127.0.0.1".parse().unwrap(), 12) {
            Err(InvalidRangeError::InvalidHostAddress(_)) => {}
            _ => panic!("Expected InvalidHostAddress failure"),
        }

        match IpRange::new("127.0.0.0".parse().unwrap(), 99) {
            Err(InvalidRangeError::InvalidPrefixLength(_)) => {}
            _ => panic!("Expected InvalidPrefixLength failure"),
        }

        match IpRange::new("127.0.0.0".parse().unwrap(), 0) {
            Err(InvalidRangeError::InvalidHostAddress(_)) => {}
            _ => panic!("Expected InvalidHostAddress failure"),
        }
    }

    #[test]
    fn test_parsing_ipv4() {
        "0.0.0.0/0".parse::<IpRange>().unwrap();
        "127.0.0.0/12".parse::<IpRange>().unwrap();
        "127.0.8.0/21".parse::<IpRange>().unwrap();

        match "127.0.0.1/12".parse::<IpRange>() {
            Err(RangeParseError::InvalidHostAddress(_)) => {}
            _ => panic!("Expected InvalidHostAddress failure"),
        }

        match "127.0.0.0/99".parse::<IpRange>() {
            Err(RangeParseError::InvalidPrefixLength(_)) => {}
            _ => panic!("Expected InvalidPrefixLength failure"),
        }

        match "127.0.0.0/0".parse::<IpRange>() {
            Err(RangeParseError::InvalidHostAddress(_)) => {}
            _ => panic!("Expected InvalidHostAddress failure"),
        }

        match "127.0.0.0x12".parse::<IpRange>() {
            Err(RangeParseError::UnparseableRange(_)) => {}
            _ => panic!("Expected UnparseableRange failure"),
        }

        match "127.0.0x0/12".parse::<IpRange>() {
            Err(RangeParseError::UnparseableRange(_)) => {}
            _ => panic!("Expected UnparseableRange failure"),
        }

        match "127.0.0.0/x".parse::<IpRange>() {
            Err(RangeParseError::UnparseableRange(_)) => {}
            _ => panic!("Expected UnparseableRange failure"),
        }

        match "127.0.0.0/".parse::<IpRange>() {
            Err(RangeParseError::UnparseableRange(_)) => {}
            _ => panic!("Expected UnparseableRange failure"),
        }
    }

    #[test]
    fn test_new_ipv6() {
        IpRange::new("::".parse().unwrap(), 0).unwrap();
        IpRange::new("::".parse().unwrap(), 125).unwrap();
        IpRange::new("::8".parse().unwrap(), 125).unwrap();

        match IpRange::new("::1".parse().unwrap(), 12) {
            Err(InvalidRangeError::InvalidHostAddress(_)) => {}
            _ => panic!("Expected InvalidHostAddress failure"),
        }

        match IpRange::new("::".parse().unwrap(), 129) {
            Err(InvalidRangeError::InvalidPrefixLength(_)) => {}
            _ => panic!("Expected InvalidPrefixLength failure"),
        }

        match IpRange::new("::8".parse().unwrap(), 0) {
            Err(InvalidRangeError::InvalidHostAddress(_)) => {}
            _ => panic!("Expected InvalidHostAddress failure"),
        }
    }

    #[test]
    fn test_parsing_ipv6() {
        "::/0".parse::<IpRange>().unwrap();
        "::/125".parse::<IpRange>().unwrap();
        "::8/125".parse::<IpRange>().unwrap();

        match "::1/12".parse::<IpRange>() {
            Err(RangeParseError::InvalidHostAddress(_)) => {}
            _ => panic!("Expected InvalidHostAddress failure"),
        }

        match "::/129".parse::<IpRange>() {
            Err(RangeParseError::InvalidPrefixLength(_)) => {}
            _ => panic!("Expected InvalidPrefixLength failure"),
        }

        match "::8/1".parse::<IpRange>() {
            Err(RangeParseError::InvalidHostAddress(_)) => {}
            _ => panic!("Expected InvalidHostAddress failure"),
        }

        match "::8x125".parse::<IpRange>() {
            Err(RangeParseError::UnparseableRange(_)) => {}
            _ => panic!("Expected UnparseableRange failure"),
        }

        match "::x/125".parse::<IpRange>() {
            Err(RangeParseError::UnparseableRange(_)) => {}
            _ => panic!("Expected UnparseableRange failure"),
        }

        match "::8/x".parse::<IpRange>() {
            Err(RangeParseError::UnparseableRange(_)) => {}
            _ => panic!("Expected UnparseableRange failure"),
        }

        match "::8/".parse::<IpRange>() {
            Err(RangeParseError::UnparseableRange(_)) => {}
            _ => panic!("Expected UnparseableRange failure"),
        }
    }
}
