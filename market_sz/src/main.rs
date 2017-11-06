// This project is written by kimikan
//@2017
//it's about a full feature shenzhen stock market parser
//mit licensed, 

//attention:
//if want to use it in live env
//just implement the todo: information
//integrated with messaging system. or something
extern crate encoding;
extern crate byteorder;

mod utils;

use std::io;
use std::fs::{OpenOptions, File};
use std::io::{BufReader, BufRead, Read};
use std::collections::HashMap;
use std::time::SystemTime;
use std::net::TcpStream;

use byteorder::ByteOrder;

const SENDER : &str = "F000648Q0011";
const TARGET : &str = "VDE";
const HEARTBEAT_INTERVAL : u32 = 20;
const PASSWORD : &str = "F000648Q0011";
const APPVER : &str = "1.00";

#[derive(Debug, Clone)]
struct Snapshot {
    _orig_time : i64,
    _channel_no :u16,
    _md_stream_id : [u8; 3],
    _security_id : [u8;8],
    _security_id_source : [u8; 4], //102 shenzhen, 103 hongkong
    _trading_phase_code : [u8; 8],
    _prev_close_px : i64,
    _num_trades : i64,
    _total_vol_trade : i64,
    _total_value_trade : i64,
}

#[derive(Debug, Clone)]
struct StockEntry {
    _entry_type : [u8; 2],
    _entry_px : i64,
    _entry_size : i64,
    _price_level : u64,
    _num_of_orders : i64,
    //_no_orders : u32,
    _orders : Vec<i64>,
}

#[derive(Debug, Clone)]
struct Stock {
    _snap_shot : Snapshot,

    _entries : Vec<StockEntry>,
}

#[derive(Debug, Clone)]
struct IndexEntry {
    _entry_type : [u8; 2],
    _entry_px : i64,
}

#[derive(Debug, Clone)]
struct Index {
    _snap_shot : Snapshot,
    _entries : Vec<IndexEntry>,
}

#[derive(Debug, Clone)]
struct MsgHead {
    _msg_type : u32,
    _body_length : u32,
}

use std::io::ErrorKind;
fn generate_checksum(bs : &[u8])->u32 {
    let mut sum : u32 = 0;
    for i in 0..bs.len() {
        sum += bs[i] as u32;
    }

    sum
}

#[derive(Debug, Clone)]
struct Context {

    _stocks : HashMap<String, Stock>,
    _indexs : HashMap<String, Index>,
}

use std::marker::Sized;
use std::slice;
use std::mem;

fn heartbeat(stream:&mut TcpStream)->io::Result<()>{
    let mut msg : [u8;12] = [0;12];

    byteorder::BigEndian::write_u32(&mut msg[..], 3);
    let checksum = generate_checksum(&msg[0..8]);
    byteorder::BigEndian::write_u32(&mut msg[8..12], checksum);

    let size = stream.write(&msg[..])?;
    if size != 12 {
        return Err(Error::from(ErrorKind::WriteZero));
    }
    println!("Heartbeat");
    Ok(())
}

fn struct_to_bytes<T:Sized>(p : &mut T)->&mut[u8] {
    unsafe {
        slice::from_raw_parts_mut((p as *mut T) as *mut u8, mem::size_of::<T>())
    }
}

use std::io::Write;
use std::io::Error;

impl Context {
    fn new()->Context {
        Context {
            _stocks : Default::default(),
            _indexs : Default::default(),
        }
    }

    fn login(&self, stream : &mut TcpStream)->io::Result<()> {
        /*
        #[repr(C, packed)]
        struct Msg {
            _msg_header : MsgHead,
            _sender : [u8;20],
            _target : [u8;20],
            _heart_beat : u32,
            _password : [u8;16],
            _version : [u8;32],
            _checksum : u32,
        }
        let mut msg  = Msg {
            _msg_header : MsgHead{
                _msg_type : 1,
                _body_length : 0,
            },
            _sender : [0; 20],
            _target : [0;20],
            _heart_beat : 30,
            _password : [0;16],
            _version : [0; 32],
            _checksum : 0,
        };*/
        let mut msg : [u8;104] = [0;104];
        
        byteorder::BigEndian::write_u32(&mut msg[..], 1);
        byteorder::BigEndian::write_u32(&mut msg[4..], 92);//len
        
        use std::cmp;
        &(msg[8..( 8 + cmp::min(20, SENDER.len()))]).copy_from_slice(SENDER.as_bytes());
        &(msg[28..( 28 + cmp::min(20, TARGET.len()))]).copy_from_slice(TARGET.as_bytes());
        byteorder::BigEndian::write_u32(&mut msg[48..], HEARTBEAT_INTERVAL);//heartbeat
        &(msg[52..( 52 + cmp::min(20, PASSWORD.len()))]).copy_from_slice(PASSWORD.as_bytes());
        &(msg[68..( 68 + cmp::min(20, APPVER.len()))]).copy_from_slice(APPVER.as_bytes());
        let checksum = generate_checksum(&msg[0..100]);
        byteorder::BigEndian::write_u32(&mut msg[100..], checksum);

        let size = stream.write(&msg[0..104])?;

        if size == 104 {
            return Ok(());
        }

        //byteorder::BigEndian::
        return Err(Error::from(ErrorKind::InvalidData));
    }

    fn get_message(&self, stream : &mut TcpStream) -> (io::Result<u32>, Option<Vec<u8>>) {
        let mut header : [u8;8] = [0u8;8];
        match stream.read_exact(&mut header[0..8]) {
            Ok(_)=>{
                let msg_type = byteorder::BigEndian::read_u32(&header[..]);
                let body_len = byteorder::BigEndian::read_u32(&header[4..]) as usize;
                
                if body_len > 0 {
                    let mut vec : Vec<u8> = Vec::with_capacity(body_len);
                    unsafe {
                        vec.set_len(body_len);
                    }

                    match stream.read_exact(&mut vec) {
                        Ok(_)=>{
                            let mut checksum:[u8;4] = [0;4];
                            let checksum_res = stream.read_exact(&mut checksum[..]);
                            if let Err(e) = checksum_res {
                                return (Err(e), None);
                            }

                            return (Ok(msg_type), Some(vec));    
                        },
                        Err(e)=>{
                            return (Err(e), None);
                        }
                    };
                } else {
                    return (Ok(msg_type), None); 
                }
            },
            Err(e)=>{
                return (Err(e), None);
            }
        };

    }

    fn handle_resent_message(&self, stream:&mut TcpStream, buf : &Vec<u8>)->io::Result<()>{

        Ok(())
    }

    fn handle_user_report_message(&self, stream:&mut TcpStream, buf : &Vec<u8>)->io::Result<()>{
        #[derive(Debug)]
        struct report_msg {
            _time : i64,
            _version_code : [u8; 16],
            _user_num : u16,
        }
        let mut msg = report_msg {
            _time : 0i64,
            _version_code : [0; 16],
            _user_num : 0u16,
        };
        
        let mut time : i64;
        let mut user_num : u16;
        { //local variables
            let mut buf = struct_to_bytes(&mut msg);
            time = byteorder::BigEndian::read_i64(&buf[..]);
            user_num = byteorder::BigEndian::read_u16(&buf[24..]);
        }
        msg._time = time;
        msg._user_num = user_num;

        println!("UserReport: {:?}", msg);
        Ok(())
    }

    fn handle_channel_statistic(&self, stream:&mut TcpStream, buf : &Vec<u8>)->io::Result<()>{

        Ok(())
    }

    fn handle_market_status_message(&self, stream:&mut TcpStream, buf : &Vec<u8>)->io::Result<()>{

        Ok(())
    }

    fn handle_realtime_status(&self, stream:&mut TcpStream, buf : &Vec<u8>)->io::Result<()>{
        #[derive(Debug)]
        struct switcher {
            _switch_type : u16,
            _switch_status : u16,
        }

        #[derive(Debug)]
        struct realtime_status {
            _time : i64,
            _channel_no : u16,
            _security_id : [u8; 8],
            _security_id_source : [u8;4],//102 shenzhen, 103 hongkong
            _financial_status : [u8; 8],
            _switchers : Vec<switcher>,
        }
        let mut msg = realtime_status {
            _time : 0i64,
            _channel_no : 0u16,
            _security_id : [0; 8],
            _security_id_source : [0; 4],
            _financial_status : [0; 8],
            _switchers : vec![],
        };
        let mut buf:[u8; 34] = [0; 34];
        stream.read_exact(&mut buf[..])?;
        msg._time = byteorder::BigEndian::read_i64(&buf[..]);
        msg._channel_no = byteorder::BigEndian::read_u16(&buf[8..]);
        (&mut msg._security_id[..]).copy_from_slice(&buf[10..18]);
        (&mut msg._security_id_source[..]).copy_from_slice(&buf[18..22]);
        (&mut msg._financial_status[..]).copy_from_slice(&buf[22..30]);

        let count = byteorder::BigEndian::read_u32(&buf[30..34]);

        for i in 0..count {
            let mut s = switcher {
                _switch_type : 0u16,
                _switch_status : 0u16,
            };
            let mut buf2 : [u8;4] = [0;4];
            stream.read_exact(&mut buf2[..])?;
            s._switch_type = byteorder::BigEndian::read_u16(&buf2[..]);
            s._switch_status = byteorder::BigEndian::read_u16(&buf2[2..]);

            msg._switchers.push(s);
        }

        println!("Realtime: {:?}", msg);
        Ok(())
    }

    fn handle_stock_report(&self, stream:&mut TcpStream, buf : &Vec<u8>)->io::Result<()>{

        Ok(())
    }


    fn handle_stock_snapshot(&self, stream:&mut TcpStream, buf : &Vec<u8>)->io::Result<()>{

        Ok(())
    }

    fn handle_inidex_snapshot(&self, stream:&mut TcpStream, buf : &Vec<u8>)->io::Result<()>{

        Ok(())
    }

    fn handle_volume_statistic(&self, stream:&mut TcpStream, buf : &Vec<u8>)->io::Result<()>{

        Ok(())
    }

    fn handle_message(&self, stream :&mut TcpStream,  msg_type:u32, buf:&Vec<u8>)->io::Result<()> {

        match msg_type {
            //heartbeat
            3=>{
                heartbeat(stream)?;
            },
            //channel heartbeat
            390095=>{
                //channel heartbeat
            },
            390094=>{
                self.handle_resent_message(stream, buf)?;
            },
            390093=>{
                self.handle_user_report_message(stream, buf)?;                
            },
            390090=>{
                self.handle_channel_statistic(stream, buf)?;
            },
            //market status
            390019=>{
                self.handle_market_status_message(stream, buf)?;
            },
            //realtime status
            390013=>{
                self.handle_realtime_status(stream, buf)?;
            },
            390012=>{
                self.handle_stock_report(stream, buf)?;
            },
            300111=>{
                self.handle_stock_snapshot(stream, buf)?;
            },
            300611=>{
                //hongkong stocks
            },
            309011=>{
                self.handle_inidex_snapshot(stream, buf)?;
            },
            309111=>{
                self.handle_volume_statistic(stream, buf)?;
            },
            _=>{
                println!("{:?}:  {:?}", msg_type, buf);
            }
        };

        Ok(())
    }

    fn run(&mut self) -> io::Result<()> {
        let mut stream = TcpStream::connect("139.196.94.8:9999")?;
        
        self.login(&mut stream)?;
        println!("login success {:?}", stream);

        let mut stream2 = stream.try_clone().unwrap();
        use std::thread;
        thread::Builder::new().name("Heart beat thread".into()).spawn(move || {
            loop {
                use std::time;
                thread::sleep(time::Duration::from_secs(15));
                if let Err(e) = heartbeat(&mut stream2) {
                    println!("heartbeat failed {:?}", e);
                }
            }
            
        }).unwrap();

        loop {
            let (res, op) = self.get_message(&mut stream);
            let msg_type  = match res {
                Ok(expr)=>expr,
                Err(e)=>{
                    return Err(e);
                }
            };

            if let Some(buf) = op {
                let _ = self.handle_message(&mut stream, msg_type, &buf)?;
            }
        }
        //Ok(())
    }
}

fn main() {
    let mut ctx = Context::new();

    if let Err(e) = ctx.run() {
        println!("{:?}", e);
    }
}
