use std::{
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
};

pub struct CentralServer<'a> {
    addr: (&'a str, u16),
    buffer: Vec<u8>,
}

#[derive(Clone, Copy)]
pub enum RequestType {
    IdRequest = 1,
    BroadcastAddress = 2,
}

pub const SERVER_ADDRESS: &str = "127.0.0.1";
pub const SERVER_PORT: u16 = 8080;

impl<'a> CentralServer<'a> {
    pub fn new(addr: (&'a str, u16)) -> CentralServer<'a> {
        CentralServer {
            addr,
            buffer: vec![0; 128],
        }
    }

    pub fn start(&mut self) -> Result<(), io::Error> {
        let listener = TcpListener::bind(&self.addr)?;
        println!("Started listening at {:?}", self.addr);

        for stream in listener.incoming() {
            self.handle_client(stream?)?;
        }

        Ok(())
    }

    fn handle_client(&mut self, mut stream: TcpStream) -> Result<(), io::Error> {
        let n = stream.read(&mut self.buffer)?;
        println!("Read {n} bytes: {:?}", self.buffer);
        stream.write(&[2])?;
        println!("Write: {:?}", &[2]);

        Ok(())
    }
}
