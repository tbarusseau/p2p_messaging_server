use std::{
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
};

use crate::server::RequestType;

pub struct Client<'a> {
    server_addr: (&'a str, u16),
    own_addr: Option<(&'a str, u16)>,
    listener: Option<TcpListener>,

    id: Option<u64>,

    buffer: Vec<u8>,
}

const ID_REQUEST: &[u8] = &[RequestType::IdRequest as u8];

impl<'a> Client<'a> {
    pub fn new(addr: (&'a str, u16)) -> Client<'a> {
        Client {
            server_addr: addr,
            own_addr: None,
            listener: None,

            id: None,

            buffer: vec![0; 128],
        }
    }

    pub fn fetch_id(&mut self) -> Result<(), io::Error> {
        let mut stream = TcpStream::connect(&self.server_addr)?;

        stream.write(ID_REQUEST)?;
        println!("Write: {:?}", ID_REQUEST);
        let n = stream.read(&mut self.buffer)?;
        println!("Read {n} bytes: {:?}", self.buffer);

        Ok(())
    }

    pub fn setup_listener(&mut self) -> Result<(), io::Error> {
        let mut available_port = 0;
        for port in 8000..9000 {
            if port_is_available(port) {
                available_port = port;
                break;
            }
        }

        println!("Set up listener at 127.0.0.1:{available_port}");

        self.own_addr = Some(("127.0.0.1", available_port));
        self.listener = Some(TcpListener::bind(("127.0.0.1", available_port))?);

        Ok(())
    }

    pub fn broadcast_address(&self) -> Result<(), io::Error> {
        Ok(())
    }

    pub fn start_listening(&mut self) -> Result<(), io::Error> {
        Ok(())
    }
}

fn port_is_available(port: u16) -> bool {
    TcpListener::bind(("127.0.0.1", port)).is_ok()
}
