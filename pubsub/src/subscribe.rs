
use nanomsg::{Socket, Protocol};
use std::io;
use std::io::Read;

pub fn loop_sub(topic:&str, url:&str)->io::Result<()> {
    let mut socket = Socket::new(Protocol::Sub)?;
    socket.subscribe(topic.as_bytes())?;
    let mut endpoint = socket.connect(url)?;

    let mut msg = String::new();
    loop {
        socket.read_to_string(&mut msg)?;
        println!("Recv '{}'.", msg);
        msg.clear();
    }
    endpoint.shutdown()?;
    Ok(())
}