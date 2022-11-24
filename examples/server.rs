use std::io;

use p2p_messaging_server::server::{CentralServer, SERVER_ADDRESS, SERVER_PORT};

fn main() -> Result<(), io::Error> {
    let mut server = CentralServer::new((SERVER_ADDRESS, SERVER_PORT));
    server.start()?;

    Ok(())
}
