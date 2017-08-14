
extern crate nanomsg;

use nanomsg::{Protocol, Socket};
use std::io;
use std::io::{Read, Write};
use std::time::Duration;
use std::thread;

fn req(url:&str) ->io::Result<()> {
    let mut socket = Socket::new(Protocol::Req)?;
    let mut endpoint = socket.connect(url)?;

    let mut reply = String::new();

    loop {
        //format request and then send it to server
        let mut request = String::new();
        let stdin = io::stdin();
        println!("Please input a string: ");
        stdin.read_line(&mut request)?;

        if request.len() <= 0 {
            continue;
        }

        match socket.write_all(&request.as_bytes()[0..request.len()-1]) {
            Ok(..) => {
                //println!("---- Send '{}'.", request);
            },
            Err(err) => {
                println!("Client failed to send request '{}'.", err);
                break
            }
        }

        match socket.read_to_string(&mut reply) {
            Ok(_) => {
                println!("---- Recv '{}'.", reply);
                reply.clear()
            },
            Err(err) => {
                println!("Client failed to receive reply '{}'.", err);
                break
            }
        }
        thread::sleep(Duration::from_millis(100));
    }

    endpoint.shutdown()?;

    Ok(())
}

//let url = "tcp://127.0.0.1:5566";
fn resp(url:&str) ->io::Result<()>{
    
    let mut socket = Socket::new(Protocol::Rep)?;
    let mut endpoint = socket.bind(url)?;
    
    loop {
        let mut request = String::new();
        if let Ok(_) = socket.read_to_string(&mut request) {
            println!("++++ Recv '{}'.", request);

            let reply = format!("{} -> Reply", request);
            match socket.write_all(reply.as_bytes()) {
                Ok(..) => {
                    //println!("++++ Sent '{}'.", reply);
                },
                Err(err) => {
                    println!("Server failed to send reply '{}'.", err);
                    break;
                }
            }
            request.clear();
            thread::sleep(Duration::from_millis(40));
        } else {
            //break the message looper
            //if recv any error
            break;
        }
    }
    endpoint.shutdown()?;
    Ok(())
}

fn run_client() {
    let url = "tcp://127.0.0.1:5566";
    req(url).unwrap();
}

fn main() {
    
    thread::spawn(||{
        let url2 = "tcp://127.0.0.1:5566";
        resp(url2).unwrap();
    });

    for _ in 0..4 {
        run_client();
    } 
    
    run_client();
}
