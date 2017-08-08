
extern crate  nanomsg;

mod publish;
mod subscribe;
mod utils;

use std::thread;
use std::time::Duration;

fn test_pub_sub() {
    const URL:&'static str = "tcp://127.0.0.1:5566";

    let mut op_pub = publish::Publisher::create(URL).unwrap();

    thread::spawn(move ||{
        for _ in 0..10 {
            op_pub.pub_msg("x3", "hello".as_bytes()).unwrap();

            thread::sleep(Duration::from_secs(1));
        }

        println!("exit");
    });

    subscribe::loop_sub("x3", URL).unwrap();
}


fn main() {
    test_pub_sub();
}
