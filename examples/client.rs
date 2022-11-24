use std::io;

use p2p_messaging_server::{
    client::Client,
    server::{SERVER_ADDRESS, SERVER_PORT},
};

fn main() -> Result<(), io::Error> {
    let mut client = Client::new((SERVER_ADDRESS, SERVER_PORT));

    client.fetch_id()?;
    client.setup_listener()?;
    client.broadcast_address()?;
    client.start_listening()?;

    Ok(())
}
