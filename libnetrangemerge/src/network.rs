use cidr::{Cidr, Inet};
use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

pub(crate) const DUMMY_NETWORK: Network = Network {
    host_address: IpAddr::V6(Ipv6Addr::new(
        0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff,
    )),
    network_length: 255,
};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Network {
    host_address: IpAddr,
    network_length: u8,
}

impl Debug for Network {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.host_address, self.network_length)
    }
}

impl Display for Network {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.host_address, self.network_length)
    }
}

pub struct InvalidNetworkLengthError {
    network_length: u8,
    max_length: u8,
}

impl Debug for InvalidNetworkLengthError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Invalid network length: {}. Network length must be {} or less",
            self.network_length, self.max_length
        )
    }
}

impl Display for InvalidNetworkLengthError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

pub struct InvalidHostAddressError {
    host_address: IpAddr,
    network_length: u8,
}

impl Debug for InvalidHostAddressError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let ipinet: cidr::IpInet = cidr::Inet::new(self.host_address, self.network_length).unwrap();
        write!(
            f,
            "Invalid host address: {} is not the first address in the network: {}",
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

pub enum InvalidNetworkError {
    InvalidNetworkLength(InvalidNetworkLengthError),
    InvalidHostAddress(InvalidHostAddressError),
}

impl Debug for InvalidNetworkError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            InvalidNetworkError::InvalidNetworkLength(err) => Debug::fmt(err, f),
            InvalidNetworkError::InvalidHostAddress(err) => Debug::fmt(err, f),
        }
    }
}

impl Display for InvalidNetworkError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl Error for InvalidNetworkError {}

#[cfg(feature = "alloc")]
pub struct UnparseableNetworkError {
    text: String,
}

#[cfg(feature = "alloc")]
impl Debug for UnparseableNetworkError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Either the host or the network length portions of {} could not be parsed",
            self.text
        )
    }
}

#[cfg(not(feature = "alloc"))]
pub struct UnparseableNetworkError {}

#[cfg(not(feature = "alloc"))]
impl Debug for UnparseableNetworkError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Either the host or the network length portions of could not be parsed",
        )
    }
}

impl Display for UnparseableNetworkError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

pub enum NetworkParseError {
    InvalidNetworkLength(InvalidNetworkLengthError),
    InvalidHostAddress(InvalidHostAddressError),
    UnparseableNetwork(UnparseableNetworkError),
}

impl Debug for NetworkParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            NetworkParseError::InvalidNetworkLength(err) => Debug::fmt(err, f),
            NetworkParseError::InvalidHostAddress(err) => Debug::fmt(err, f),
            NetworkParseError::UnparseableNetwork(err) => Debug::fmt(err, f),
        }
    }
}

impl Display for NetworkParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl Error for NetworkParseError {}

impl Network {
    pub fn new(host_address: IpAddr, network_length: u8) -> Result<Network, InvalidNetworkError> {
        match host_address {
            IpAddr::V4(_) => {
                if network_length > 32 {
                    return Err(InvalidNetworkError::InvalidNetworkLength(
                        InvalidNetworkLengthError {
                            network_length,
                            max_length: 32,
                        },
                    ));
                }
            }
            IpAddr::V6(_) => {
                if network_length > 128 {
                    return Err(InvalidNetworkError::InvalidNetworkLength(
                        InvalidNetworkLengthError {
                            network_length,
                            max_length: 128,
                        },
                    ));
                }
            }
        }
        let ipinet: cidr::IpInet = cidr::Inet::new(host_address, network_length).unwrap();
        if ipinet.first_address() != host_address {
            return Err(InvalidNetworkError::InvalidHostAddress(
                InvalidHostAddressError {
                    host_address,
                    network_length,
                },
            ));
        }
        Ok(Network {
            host_address,
            network_length,
        })
    }

    pub fn host_address(&self) -> IpAddr {
        self.host_address()
    }

    pub fn network_length(&self) -> u8 {
        self.network_length()
    }

    pub(crate) fn cidr(&self) -> cidr::IpCidr {
        cidr::Cidr::new(self.host_address, self.network_length).unwrap()
    }

    pub(crate) fn contains(&self, other: Network) -> bool {
        self.cidr().contains(&other.host_address)
    }
}

#[cfg(feature = "alloc")]
fn parse_error(s: &str) -> Result<Network, NetworkParseError> {
    Err(NetworkParseError::UnparseableNetwork(
        UnparseableNetworkError {
            text: s.to_string(),
        },
    ))
}

#[cfg(not(feature = "alloc"))]
fn parse_error(s: &str) -> Result<Network, NetworkParseError> {
    Err(NetworkParseError::UnparseableNetwork(
        UnparseableNetworkError {},
    ))
}

impl FromStr for Network {
    type Err = NetworkParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let idx_of_slash = if let Some(pos) = s.find("/") {
            pos
        } else {
            return parse_error(s);
        };

        let host_part = &s[0..idx_of_slash];
        let network_length_part = &s[idx_of_slash..][1..];

        let host_address = if let Ok(host_address) = host_part.parse() {
            host_address
        } else {
            return parse_error(s);
        };

        let network_length = if let Ok(network_length) = network_length_part.parse() {
            network_length
        } else {
            return parse_error(s);
        };

        match Network::new(host_address, network_length) {
            Ok(network) => Ok(network),
            Err(InvalidNetworkError::InvalidNetworkLength(err)) => {
                Err(NetworkParseError::InvalidNetworkLength(err))
            }
            Err(InvalidNetworkError::InvalidHostAddress(err)) => {
                Err(NetworkParseError::InvalidHostAddress(err))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::network::{InvalidNetworkError, Network, NetworkParseError};

    #[test]
    fn test_new_ipv4() {
        Network::new("0.0.0.0".parse().unwrap(), 0).unwrap();
        Network::new("127.0.0.0".parse().unwrap(), 12).unwrap();
        Network::new("127.0.8.0".parse().unwrap(), 21).unwrap();

        match Network::new("127.0.0.1".parse().unwrap(), 12) {
            Err(InvalidNetworkError::InvalidHostAddress(_)) => {}
            _ => panic!("Expected InvalidHostAddress failure"),
        }

        match Network::new("127.0.0.0".parse().unwrap(), 99) {
            Err(InvalidNetworkError::InvalidNetworkLength(_)) => {}
            _ => panic!("Expected InvalidNetworkLength failure"),
        }

        match Network::new("127.0.0.0".parse().unwrap(), 0) {
            Err(InvalidNetworkError::InvalidHostAddress(_)) => {}
            _ => panic!("Expected InvalidHostAddress failure"),
        }
    }

    #[test]
    fn test_parsing_ipv4() {
        "0.0.0.0/0".parse::<Network>().unwrap();
        "127.0.0.0/12".parse::<Network>().unwrap();
        "127.0.8.0/21".parse::<Network>().unwrap();

        match "127.0.0.1/12".parse::<Network>() {
            Err(NetworkParseError::InvalidHostAddress(_)) => {}
            _ => panic!("Expected InvalidHostAddress failure"),
        }

        match "127.0.0.0/99".parse::<Network>() {
            Err(NetworkParseError::InvalidNetworkLength(_)) => {}
            _ => panic!("Expected InvalidNetworkLength failure"),
        }

        match "127.0.0.0/0".parse::<Network>() {
            Err(NetworkParseError::InvalidHostAddress(_)) => {}
            _ => panic!("Expected InvalidHostAddress failure"),
        }

        match "127.0.0.0x12".parse::<Network>() {
            Err(NetworkParseError::UnparseableNetwork(_)) => {}
            _ => panic!("Expected UnparseableNetwork failure"),
        }

        match "127.0.0x0/12".parse::<Network>() {
            Err(NetworkParseError::UnparseableNetwork(_)) => {}
            _ => panic!("Expected UnparseableNetwork failure"),
        }

        match "127.0.0.0/x".parse::<Network>() {
            Err(NetworkParseError::UnparseableNetwork(_)) => {}
            _ => panic!("Expected UnparseableNetwork failure"),
        }

        match "127.0.0.0/".parse::<Network>() {
            Err(NetworkParseError::UnparseableNetwork(_)) => {}
            _ => panic!("Expected UnparseableNetwork failure"),
        }
    }

    #[test]
    fn test_new_ipv6() {
        Network::new("::".parse().unwrap(), 0).unwrap();
        Network::new("::".parse().unwrap(), 125).unwrap();
        Network::new("::8".parse().unwrap(), 125).unwrap();

        match Network::new("::1".parse().unwrap(), 12) {
            Err(InvalidNetworkError::InvalidHostAddress(_)) => {}
            _ => panic!("Expected InvalidHostAddress failure"),
        }

        match Network::new("::".parse().unwrap(), 129) {
            Err(InvalidNetworkError::InvalidNetworkLength(_)) => {}
            _ => panic!("Expected InvalidNetworkLength failure"),
        }

        match Network::new("::8".parse().unwrap(), 0) {
            Err(InvalidNetworkError::InvalidHostAddress(_)) => {}
            _ => panic!("Expected InvalidHostAddress failure"),
        }
    }

    #[test]
    fn test_parsing_ipv6() {
        "::/0".parse::<Network>().unwrap();
        "::/125".parse::<Network>().unwrap();
        "::8/125".parse::<Network>().unwrap();

        match "::1/12".parse::<Network>() {
            Err(NetworkParseError::InvalidHostAddress(_)) => {}
            _ => panic!("Expected InvalidHostAddress failure"),
        }

        match "::/129".parse::<Network>() {
            Err(NetworkParseError::InvalidNetworkLength(_)) => {}
            _ => panic!("Expected InvalidNetworkLength failure"),
        }

        match "::8/1".parse::<Network>() {
            Err(NetworkParseError::InvalidHostAddress(_)) => {}
            _ => panic!("Expected InvalidHostAddress failure"),
        }

        match "::8x125".parse::<Network>() {
            Err(NetworkParseError::UnparseableNetwork(_)) => {}
            _ => panic!("Expected UnparseableNetwork failure"),
        }

        match "::x/125".parse::<Network>() {
            Err(NetworkParseError::UnparseableNetwork(_)) => {}
            _ => panic!("Expected UnparseableNetwork failure"),
        }

        match "::8/x".parse::<Network>() {
            Err(NetworkParseError::UnparseableNetwork(_)) => {}
            _ => panic!("Expected UnparseableNetwork failure"),
        }

        match "::8/".parse::<Network>() {
            Err(NetworkParseError::UnparseableNetwork(_)) => {}
            _ => panic!("Expected UnparseableNetwork failure"),
        }
    }
}
