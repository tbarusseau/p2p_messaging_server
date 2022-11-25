use std::{
    io::{stdin, stdout, Write},
    sync::mpsc::channel,
    thread,
};

use p2p_messaging_server::{
    client::{Client, ClientError},
    server::{SERVER_ADDRESS, SERVER_PORT},
};

fn main() -> Result<(), ClientError> {
    let mut client = Client::new((SERVER_ADDRESS, SERVER_PORT));

    let mut listener = client.setup_listener()?;
    client.set_default_id()?;
    client.broadcast()?;

    let (tx, rx) = channel();

    thread::spawn(move || Client::start_listening(&mut listener, tx.clone()));

    // Wait for print to be done in thread
    rx.recv().unwrap();

    loop {
        let mut target = String::new();
        let mut message = String::new();

        println!("--> Sending a new message. Press enter to send input.");
        print!(" -> Target: ");
        stdout().flush().unwrap();
        stdin().read_line(&mut target).unwrap();

        print!(" -> Message: ");
        stdout().flush().unwrap();
        stdin().read_line(&mut message).unwrap();

        target = target.trim().to_owned();
        message = message.trim().to_owned();

        client.send_message(target, message)?;
    }
}
