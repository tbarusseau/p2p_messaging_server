use std::{
    collections::HashMap,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    sync::mpsc::Sender,
};

use crate::{get_data, port_is_available, server::ServerMessageType, Addr, Identifier};

#[derive(Debug)]
pub enum ClientError {
    AddrNotSet,
    IdNotSet,
    InvalidServerResponse,
    InvalidClientResponse,
    IoError(io::Error),
}

impl From<io::Error> for ClientError {
    fn from(e: io::Error) -> Self {
        ClientError::IoError(e)
    }
}

/// A P2P client that needs the centralized server to get addresses, and then bypasses it.
pub struct Client<'a> {
    server_addr: (&'a str, u16),
    own_addr: Option<(&'a str, u16)>,

    id: Option<Identifier>,
    index: HashMap<Identifier, Addr>,

    buffer: Vec<u8>,
}

#[derive(Clone, Copy)]
pub enum ClientMessageType {
    BroadcastId = 1,
    Message = 2,
    GetClientAddr = 3,
    Ack = 4,
}

impl<'a> Client<'a> {
    pub fn new(addr: (&'a str, u16)) -> Client<'a> {
        Client {
            server_addr: addr,
            own_addr: None,

            id: None,
            index: HashMap::new(),

            buffer: vec![0; 128],
        }
    }

    pub fn set_default_id(&mut self) -> Result<(), ClientError> {
        if self.own_addr.is_none() {
            return Err(ClientError::AddrNotSet);
        }

        let addr = self.own_addr.as_ref().unwrap();
        self.id = Some(format!("client:{}", addr.1));

        println!("Client id: {}", self.id.as_ref().unwrap());

        Ok(())
    }

    pub fn set_id(&mut self, id: Identifier) {
        self.id = Some(id);

        println!("Client id: {}", self.id.as_ref().unwrap());
    }

    pub fn setup_listener(&mut self) -> Result<TcpListener, ClientError> {
        let mut available_port = 0;
        for port in 8000..9000 {
            println!("Checking port availability: {port}");
            if port_is_available(port) {
                available_port = port;
                break;
            }
        }

        println!("Set up listener at 127.0.0.1:{available_port}");
        self.own_addr = Some(("127.0.0.1", available_port));

        Ok(TcpListener::bind(("127.0.0.1", available_port))?)
    }

    pub fn broadcast(&mut self) -> Result<(), ClientError> {
        let mut stream = TcpStream::connect(self.server_addr)?;

        self.broadcast_id(&mut stream)
    }

    fn broadcast_id(&mut self, stream: &mut TcpStream) -> Result<(), ClientError> {
        if self.id.is_none() {
            return Err(ClientError::IdNotSet);
        }

        if self.own_addr.is_none() {
            return Err(ClientError::AddrNotSet);
        }

        let id = self.id.as_ref().unwrap();
        // Message type
        let mut message = vec![ClientMessageType::BroadcastId as u8];
        // Id
        message.append(&mut id.as_bytes().to_vec());
        // Separator
        message.push(0);
        // Own addr
        let own_addr = self.own_addr.unwrap();
        message.append(&mut own_addr.0.as_bytes().to_vec());
        message.push(b':');
        message.append(&mut own_addr.1.to_be_bytes().to_vec());

        stream.write(&message)?;

        self.check_ack(stream)
    }

    pub fn send_message(&mut self, target: String, message: String) -> Result<(), ClientError> {
        self.buffer = vec![0; 128];

        let addr = self.get_client_addr(&target)?;

        let mut stream = TcpStream::connect(addr)?;

        let mut msg = vec![ClientMessageType::Message as u8];
        // Message
        msg.append(&mut message.as_bytes().to_vec());

        stream.write(&msg)?;

        self.check_client_ack(&mut stream)
    }

    fn get_client_addr(&mut self, target: &str) -> Result<Addr, ClientError> {
        if self.index.contains_key(target) {
            println!("  [ Getting target addr from index ]");
            return Ok(self.index.get(target).unwrap().clone());
        }

        println!("  [ Fetching target addr from centralized server ]");

        let mut msg = vec![ClientMessageType::GetClientAddr as u8];
        // Target
        msg.append(&mut target.as_bytes().to_vec());

        let mut stream = TcpStream::connect(self.server_addr)?;
        stream.write(&msg)?;
        self.check_ack(&mut stream)?;

        let addr = get_data(&self.buffer, 1, |b| *b != b':');
        let port = get_data(&self.buffer, 2 + addr.len(), |b| *b != 0);

        let addr = String::from_utf8(addr).map_err(|_| ClientError::InvalidServerResponse)?;
        let port: [u8; 2] = [port[0], port[1]];
        let port = u16::from_be_bytes(port);

        println!("  [ Received address: {}:{} ]", addr, port);

        self.index
            .insert(target.to_owned(), Addr(addr.as_bytes().to_owned(), port));

        Ok(Addr(addr.as_bytes().to_vec(), port))
    }

    pub fn start_listening(listener: &mut TcpListener, tx: Sender<()>) -> Result<(), ClientError> {
        println!("Start listening at addr {}", listener.local_addr().unwrap());
        tx.send(()).unwrap();
        let mut buffer = vec![0; 128];

        for stream in listener.incoming() {
            let mut stream = stream?;
            let n = stream.read(&mut buffer)?;

            if n == 0 || n > 127 {
                panic!()
            }

            if let Some(request_type) = buffer.first() {
                match request_type {
                    n if *n == ClientMessageType::Message as u8 => {
                        Client::handle_client_message(&buffer, &mut stream)?;
                    }
                    _ => panic!(),
                }
            }
        }

        Ok(())
    }

    fn handle_client_message(buffer: &[u8], stream: &mut TcpStream) -> Result<(), ClientError> {
        let msg = get_data(buffer, 1, |b| *b != 0);
        let msg = String::from_utf8(msg).map_err(|_| ClientError::InvalidClientResponse)?;

        println!("<-- Received message: {msg}");

        let buffer = vec![ClientMessageType::Ack as u8];
        stream.write(&buffer)?;

        Ok(())
    }

    fn check_ack(&mut self, stream: &mut TcpStream) -> Result<(), ClientError> {
        self.buffer = vec![0; 128];
        let n = stream.read(&mut self.buffer)?;

        if n == 0 || n > 127 || self.buffer[0] != ServerMessageType::Ack as u8 {
            Err(ClientError::InvalidServerResponse)
        } else {
            Ok(())
        }
    }

    fn check_client_ack(&mut self, stream: &mut TcpStream) -> Result<(), ClientError> {
        let n = stream.read(&mut self.buffer)?;

        if n == 0 || n > 127 || self.buffer[0] != ClientMessageType::Ack as u8 {
            Err(ClientError::InvalidClientResponse)
        } else {
            Ok(())
        }
    }
}
