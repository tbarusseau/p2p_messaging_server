use std::{
    fmt::Display,
    net::{SocketAddr, ToSocketAddrs},
    vec,
};

pub mod client;
pub mod server;

pub type Identifier = String;

#[derive(Debug, Clone)]
pub struct Addr(pub Vec<u8>, pub u16);

impl Display for Addr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", String::from_utf8_lossy(self.0.as_ref()), self.1)
    }
}

impl ToSocketAddrs for Addr {
    type Iter = vec::IntoIter<SocketAddr>;

    fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
        (String::from_utf8_lossy(self.0.as_ref()).as_ref(), self.1).to_socket_addrs()
    }
}

impl From<SocketAddr> for Addr {
    fn from(s: SocketAddr) -> Self {
        println!("Port: {}", s.port());

        match s.ip() {
            std::net::IpAddr::V4(v4) => Addr(v4.to_string().as_bytes().to_vec(), s.port()),
            std::net::IpAddr::V6(_) => panic!("IPv6 not supported"),
        }
    }
}

fn port_is_available(port: u16) -> bool {
    std::net::TcpListener::bind(("127.0.0.1", port)).is_ok()
}

fn get_data<P: FnMut(&u8) -> bool>(buffer: &[u8], skip: usize, predicate: P) -> Vec<u8> {
    buffer
        .iter()
        .cloned()
        .skip(skip)
        .take_while(predicate)
        .collect()
}
