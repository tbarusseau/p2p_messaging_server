use std::{
    collections::HashMap,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
};

use crate::{client::ClientMessageType, get_data, Addr, Identifier};

pub struct CentralServer {
    addr: Addr,
    buffer: Vec<u8>,

    index: HashMap<Identifier, Addr>,
}

#[derive(Debug)]
pub enum ServerError {
    InvalidIdentifier,
    InvalidClientAddress,
    InvalidClientPort,
    InvalidClientTarget,
    InvalidClientMessage,
    IoError(io::Error),
}

impl From<io::Error> for ServerError {
    fn from(e: io::Error) -> Self {
        ServerError::IoError(e)
    }
}

#[derive(Clone, Copy)]
pub enum ServerMessageType {
    Ack = 1,
}

pub const SERVER_ADDRESS: &str = "127.0.0.1";
pub const SERVER_PORT: u16 = 8080;

impl CentralServer {
    pub fn new(addr: Addr) -> CentralServer {
        CentralServer {
            addr,
            buffer: vec![0; 128],
            index: HashMap::new(),
        }
    }

    pub fn start(&mut self) -> Result<(), ServerError> {
        let listener = TcpListener::bind(&self.addr)?;
        println!("Started listening at {}", self.addr);

        for stream in listener.incoming() {
            self.buffer = vec![0; 128];
            self.handle_client(stream?)?;
        }

        Ok(())
    }

    fn handle_client(&mut self, mut stream: TcpStream) -> Result<(), ServerError> {
        let n = stream.read(&mut self.buffer)?;

        if n == 0 || n > 127 {
            panic!()
        }

        if let Some(request_type) = self.buffer.first() {
            match request_type {
                n if *n == ClientMessageType::BroadcastId as u8 => {
                    self.handle_broadcast_id_request(&mut stream)?;
                }
                n if *n == ClientMessageType::GetClientAddr as u8 => {
                    self.handle_get_client_addr_request(&mut stream)?;
                }
                _ => panic!(),
            }
        }

        Ok(())
    }

    fn handle_broadcast_id_request(&mut self, stream: &mut TcpStream) -> Result<(), ServerError> {
        let id = get_data(&self.buffer, 1, |b| *b != 0);
        let addr = get_data(&self.buffer, id.len() + 2, |b| *b != b':');
        let port = get_data(&self.buffer, id.len() + addr.len() + 3, |b| *b != 0);

        let id: String = String::from_utf8(id).map_err(|_| ServerError::InvalidIdentifier)?;
        let addr = String::from_utf8(addr).map_err(|_| ServerError::InvalidClientAddress)?;
        let port: [u8; 2] = [port[0], port[1]];
        let port = u16::from_be_bytes(port);

        println!("<-- Received addr for client {}: {}:{}", id, addr, port);

        self.index
            .insert(id, Addr(addr.as_bytes().to_owned(), port));

        stream.write(&[ServerMessageType::Ack as u8])?;

        Ok(())
    }

    fn handle_get_client_addr_request(
        &mut self,
        stream: &mut TcpStream,
    ) -> Result<(), ServerError> {
        let id = get_data(&self.buffer, 1, |b| *b != 0);
        let id = String::from_utf8(id).map_err(|_| ServerError::InvalidClientTarget)?;

        if self.index.contains_key(&id) {
            let addr = self.index.get(&id).unwrap();

            println!("--> Sending addr \"{addr}\"");

            // Ack
            let mut msg = vec![ServerMessageType::Ack as u8];
            // Addr
            msg.append(&mut (addr.0.clone()));
            msg.push(b':');
            msg.append(&mut (addr.1.to_be_bytes().to_vec()));

            stream.write(&msg)?;
        }

        Ok(())
    }
}
