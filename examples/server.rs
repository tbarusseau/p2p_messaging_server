use p2p_messaging_server::{
    server::{CentralServer, ServerError, SERVER_ADDRESS, SERVER_PORT},
    Addr,
};

fn main() -> Result<(), ServerError> {
    let mut server = CentralServer::new(Addr(SERVER_ADDRESS.as_bytes().to_vec(), SERVER_PORT));
    server.start()?;

    Ok(())
}
