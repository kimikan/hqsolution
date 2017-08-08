
use std::io;
use nanomsg::{Socket, Protocol, Endpoint};

use std::io::Write;

pub struct Publisher {
    //start for definition
    _url:String,

    //server socket to write into
    _sock:Socket,

    //endpoint, server side......
    _end_point:Endpoint,
}

//publisher
impl Publisher {

    //create a new publisher
    pub fn create(url:&str)->Option<Publisher> {

        let socket = Socket::new(Protocol::Pub);

        if let Ok(mut s) = socket {
            let endpoint = match s.bind(url) {
                Ok(t)=>t,
                Err(_)=>{
                    return None;
                }
            };

            let p = Publisher {
                _url:url.to_owned(),
                _sock:s,
                _end_point:endpoint,
            };

            return Some(p);
        }

        None
    }

    //Implementation close
    pub fn close(&mut self) {
        self._end_point.shutdown();
    }

    //pub message schedule a new loop
    pub fn pub_msg(&mut self, topic:&str, buf:&[u8])->io::Result<()> {

        let mut msg = Vec::with_capacity(topic.len() + buf.len() + 1);
        msg.clear();
        msg.extend_from_slice(topic.as_bytes());
        msg.extend_from_slice(buf);

        self._sock.write_all(&msg)?;
        Ok(())
    }
}

impl Drop for Publisher {

    fn drop(&mut self){
        self.close();
    }
}