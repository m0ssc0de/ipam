extern crate ipnetwork;
use std::convert::TryInto;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
enum Error {
    CheckOffsetError(std::num::TryFromIntError),
    OffsetTooBig,
}

impl From<std::num::TryFromIntError> for Error {
    fn from(err: std::num::TryFromIntError) -> Error {
        Error::CheckOffsetError(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::OffsetTooBig => write!(f, "Offset is bigger than size of networking"),
            Error::CheckOffsetError(ref t) => t.fmt(f),
        }
    }
}

struct IpPool {
    net_iter: std::iter::Skip<ipnetwork::IpNetworkIterator>,
}

impl IpPool {
    fn new(net: ipnetwork::IpNetwork, offset: usize) -> Result<IpPool, Error> {
        match net.size() {
            ipnetwork::NetworkSize::V4(s) => {
                if offset > s.try_into()? {
                    return Err(Error::OffsetTooBig);
                }
            }
            ipnetwork::NetworkSize::V6(s) => {
                if offset > s.try_into()? {
                    return Err(Error::OffsetTooBig);
                }
            }
        }
        Ok(IpPool {
            net_iter: net.into_iter().skip(offset),
        })
    }
    // TODO
    fn new_addr(&mut self) -> Option<std::net::IpAddr> {
        self.net_iter.next()
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let net: ipnetwork::IpNetwork = "192.168.100.1/24".parse().unwrap();
        let mut p = crate::IpPool::new(net, 253).unwrap();
        match p.new_addr() {
            Some(addr) => {
                assert_eq!("192.168.100.253".parse::<std::net::IpAddr>().unwrap(), addr);
            }
            // TODO
            None => {}
        }

        let net: ipnetwork::IpNetwork = "192.168.100.1/24".parse().unwrap();
        match crate::IpPool::new(net, 256) {
            Ok(_) => {
                // TODO error
            }
            Err(e) => assert_eq!(crate::Error::OffsetTooBig, e),
        }
    }
}
