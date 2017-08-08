
use nanomsg::{Socket, Protocol};
use std::io;
use std::io::Read;

pub fn loop_sub(topic:&str, url:&str)->io::Result<()> {
    let mut socket = Socket::new(Protocol::Sub)?;
    let setopt = socket.subscribe(topic.as_bytes());
    let mut endpoint = socket.connect(url)?;

    match setopt {
        Ok(_) => println!("Subscribed to '{:?}'.", topic),
        Err(err) => println!("Client failed to subscribe '{}'.", err)
    }

    let mut msg = String::new();
    loop {
        match socket.read_to_string(&mut msg) {
            Ok(_) => {
                println!("Recv '{}'.", msg);
                msg.clear()
            },
            Err(err) => {
                println!("Client failed to receive msg '{}'.", err);
                break
            }
        }
    }

    endpoint.shutdown().unwrap();

    Ok(())
}