// This project is written by kimikan
//@2017
//it's about a full feature shenzhen stock market parser
//mit licensed, 

//attention:
//if want to use it in live env
//just implement the todo: information
//integrated with messaging system. or something

#![allow(unused_imports)]

#[macro_use()]
extern crate log;

#[macro_use]
extern crate tera;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

extern crate encoding;
extern crate byteorder;

mod utils;
use log::*;

use std::io;
use std::io::{Read};
use std::collections::HashMap;
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

impl Default for Snapshot {

    fn default()->Snapshot {
        Snapshot {
            _orig_time : 0i64,
            _channel_no : 0u16,
            _md_stream_id : [0; 3],
            _security_id : [0; 8],
            _security_id_source : [0; 4],
            _trading_phase_code : [0; 8],
            _prev_close_px : 0i64,
            _num_trades : 0i64,
            _total_vol_trade : 0i64,
            _total_value_trade : 0i64,
        }
    }
}

#[derive(Debug, Clone)]
struct StockEntry {
    _entry_type : [u8; 2],
    _entry_px : i64,
    _entry_size : i64,
    _price_level : u16,
    _num_of_orders : i64,
    //_no_orders : u32,
    _orders : Vec<i64>,
}

impl Default for StockEntry {
    
    fn default()->StockEntry {
        StockEntry {
            _entry_type : [0; 2],
            _entry_px : 0i64,
            _entry_size : 0i64,
            _price_level : 0u16,
            _num_of_orders : 0i64,

            _orders : vec![],
        }
    }
}

#[derive(Debug, Clone)]
struct Stock {
    _snap_shot : Snapshot,

    _entries : Vec<StockEntry>,
}

impl Default for Stock {
    fn default()->Stock {
        Stock {
            _snap_shot : Default::default(),
            _entries : vec![],
        }
    }
}

#[derive(Debug, Clone)]
struct IndexEntry {
    _entry_type : [u8; 2],
    _entry_px : i64,
}

impl Default for IndexEntry {

    fn default() -> IndexEntry {
        IndexEntry {
            _entry_type : [0; 2],
            _entry_px : 0i64,
        }
    }
}

#[derive(Debug, Clone)]
struct Index {
    _snap_shot : Snapshot,
    _entries : Vec<IndexEntry>,
}

impl Default for Index {

    fn default()->Index {
        Index {
            _snap_shot : Default::default(),
            _entries : vec![],
        }
    }
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

//msgtype 3, with a 0 len body
fn heartbeat(stream:&mut TcpStream)->io::Result<()>{
    let mut msg : [u8;12] = [0;12];

    byteorder::BigEndian::write_u32(&mut msg[..], 3);
    let checksum = generate_checksum(&msg[0..8]);
    byteorder::BigEndian::write_u32(&mut msg[8..12], checksum);

    let size = stream.write(&msg[..])?;
    if size != 12 {
        return Err(Error::from(ErrorKind::WriteZero));
    }
    info!("Heartbeat");
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
        }*/
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

    //message header + message body + checksum
    fn get_message(&self, stream : &mut TcpStream) -> (io::Result<u32>, Option<Vec<u8>>) {
        let mut header : [u8;8] = [0u8;8];

        let res = stream.read_exact(&mut header[0..8]);
        if let Err(e) = res {
            return (Err(e), None);
        }
        
        let msg_type = byteorder::BigEndian::read_u32(&header[..]);
        let body_len = byteorder::BigEndian::read_u32(&header[4..]) as usize;
        
        if body_len <= 0 {
            return (Ok(msg_type), None);
        }

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
    }

    fn handle_resent_message(&self, _:&mut TcpStream, _ : &Vec<u8>)->io::Result<()>{

        Ok(())
    }

    fn handle_user_report_message(&self, _:&mut TcpStream, _ : &Vec<u8>)->io::Result<()>{
        
        #[derive(Debug)]
        struct ReportMsg {
            _time : i64,
            _version_code : [u8; 16],
            _user_num : u16,
        }
        let mut msg = ReportMsg {
            _time : 0i64,
            _version_code : [0; 16],
            _user_num : 0u16,
        };
        
        let time : i64;
        let user_num : u16;
        { //local variables
            let buf = struct_to_bytes(&mut msg);
            time = byteorder::BigEndian::read_i64(&buf[..]);
            user_num = byteorder::BigEndian::read_u16(&buf[24..]);
        }
        msg._time = time;
        msg._user_num = user_num;

        info!("UserReport: {:?}", msg);
        Ok(())
    }

    fn handle_channel_statistic(&self, _ : &mut TcpStream, _ : &Vec<u8>)->io::Result<()>{

        Ok(())
    }

    fn handle_market_status_message(&self, _ : &mut TcpStream, _ : &Vec<u8>)->io::Result<()>{

        Ok(())
    }

    fn handle_realtime_status(&self, _:&mut TcpStream, buf : &Vec<u8>)->io::Result<()>{
        
        #[derive(Debug)]
        struct Switcher {
            _switch_type : u16,
            _switch_status : u16,
        }

        #[derive(Debug)]
        struct RealtimeStatus {
            _time : i64,
            _channel_no : u16,
            _security_id : [u8; 8],
            _security_id_source : [u8;4],//102 shenzhen, 103 hongkong
            _financial_status : [u8; 8],
            _switchers : Vec<Switcher>,
        }
        let mut msg = RealtimeStatus {
            _time : 0i64,
            _channel_no : 0u16,
            _security_id : [0; 8],
            _security_id_source : [0; 4],
            _financial_status : [0; 8],
            _switchers : vec![],
        };
        
        msg._time = byteorder::BigEndian::read_i64(&buf[..]);
        msg._channel_no = byteorder::BigEndian::read_u16(&buf[8..]);
        
        (&mut msg._security_id[..]).copy_from_slice(&buf[10..18]);
        (&mut msg._security_id_source[..]).copy_from_slice(&buf[18..22]);
        (&mut msg._financial_status[..]).copy_from_slice(&buf[22..30]);

        let count = byteorder::BigEndian::read_u32(&buf[30..34]);

        for i in 0..count {
            let mut s = Switcher {
                _switch_type : 0u16,
                _switch_status : 0u16,
            };
            
            s._switch_type = byteorder::BigEndian::read_u16(&buf[(34 + i * 4) as usize ..]);
            s._switch_status = byteorder::BigEndian::read_u16(&buf[(34 + i * 4  + 2)as usize..]);

            msg._switchers.push(s);
        }

        info!("Realtime: {:?}", msg);
        Ok(())
    }

    fn handle_stock_report(&self, _:&mut TcpStream, buf : &Vec<u8>)->io::Result<()>{
        #[derive(Debug)]
        struct StockReport {
            _time : i64,
            _channel_no:u16,
            _news_id : [u8; 8],
            _head_line : String,//[u8; 128],
            _raw_data_format : [u8; 8],//txt, pdf, doc
            _raw_data_length : u32,
            _raw_data : Vec<u8>,
        }
        let mut msg = StockReport {
            _time : 0i64,
            _channel_no : 0u16,
            _news_id : [0; 8],
            _head_line : "".to_owned(),//[0; 128],
            _raw_data_format : [0; 8],
            _raw_data_length : 0u32,
            _raw_data : vec![],
        };

        msg._time = byteorder::BigEndian::read_i64(&buf[..]);
        msg._channel_no = byteorder::BigEndian::read_u16(&buf[8..]);
        
        (&mut msg._news_id[..]).copy_from_slice(&buf[10..18]);
        //(&mut msg._head_line[..]).copy_from_slice(&buf[18..146]);
        let utfstr = std::str::from_utf8(&buf[18..146]);
        if let Ok(str) = utfstr {
            msg._head_line = str.to_owned();
        } else {
            info!("utftostring failed");
        }
        
        (&mut msg._raw_data_format[..]).copy_from_slice(&buf[146..154]);
        msg._raw_data_length = byteorder::BigEndian::read_u32(&buf[154..]);

        if msg._raw_data_length > 0 {
            msg._raw_data.reserve_exact(msg._raw_data_length as usize );
            unsafe {
                msg._raw_data.set_len(msg._raw_data_length as usize);
            }
            (&mut msg._raw_data[..]).copy_from_slice(&buf[158..]);
        }

        info!("Stockreport: {:?}", msg);
        Ok(())
    }

    //the main function is this one
    fn handle_stock_snapshot(&self, _ : &mut TcpStream, buf : &Vec<u8>)->io::Result<()>{
        
        let mut msg: Stock = Default::default();
        //msg._snap_shot._orig_time = byteorder::BigEndian::readi64()
        msg._snap_shot._orig_time = byteorder::BigEndian::read_i64(&buf[..]);
        msg._snap_shot._channel_no = byteorder::BigEndian::read_u16(&buf[8..]);
        (&mut msg._snap_shot._md_stream_id[..]).copy_from_slice(&buf[10..13]);
        (&mut msg._snap_shot._security_id[..]).copy_from_slice(&buf[13..21]);
        (&mut msg._snap_shot._security_id_source[..]).copy_from_slice(&buf[21..25]);
        (&mut msg._snap_shot._trading_phase_code[..]).copy_from_slice(&buf[25..33]);
        msg._snap_shot._prev_close_px = byteorder::BigEndian::read_i64(&buf[33..41]);
        msg._snap_shot._num_trades = byteorder::BigEndian::read_i64(&buf[41..49]);
        msg._snap_shot._total_vol_trade = byteorder::BigEndian::read_i64(&buf[49..57]);
        msg._snap_shot._total_value_trade = byteorder::BigEndian::read_i64(&buf[57..65]);

        let entry_count = byteorder::BigEndian::read_u32(&buf[65..69]);
        let mut start : usize = 69;
        for _ in 0..entry_count {
            let mut entry : StockEntry = Default::default();
            
            (&mut entry._entry_type[..]).copy_from_slice(&buf[start .. start + 2]);
            
            entry._entry_px = byteorder::BigEndian::read_i64(&buf[start+ 2 .. start + 10]);
            entry._entry_size = byteorder::BigEndian::read_i64(&buf[start+ 10 .. start + 18]);
            entry._price_level = byteorder::BigEndian::read_u16(&buf[start+ 18 .. start + 20]);
            entry._num_of_orders = byteorder::BigEndian::read_i64(&buf[start + 20 .. start + 28]);

            let qty_count = byteorder::BigEndian::read_u32(&buf[start + 28 .. start + 32]);

            let mut start2 = start + 32;
            for _ in 0..qty_count {

                let qty = byteorder::BigEndian::read_i64(&buf[start2 .. start2 + 8]);
                entry._orders.push(qty);
                start2 += 8;
            }

            start = start2;
            msg._entries.push(entry);
        }

        info!("Stocksnapshot: {:?}", msg);
        Ok(())
    }

    fn handle_index_snapshot(&self, _:&mut TcpStream, buf : &Vec<u8>)->io::Result<()>{

        let mut msg: Index = Default::default();
        //msg._snap_shot._orig_time = byteorder::BigEndian::readi64()
        msg._snap_shot._orig_time = byteorder::BigEndian::read_i64(&buf[..]);
        msg._snap_shot._channel_no = byteorder::BigEndian::read_u16(&buf[8..]);
        
        (&mut msg._snap_shot._md_stream_id[..]).copy_from_slice(&buf[10..13]);
        (&mut msg._snap_shot._security_id[..]).copy_from_slice(&buf[13..21]);
        (&mut msg._snap_shot._security_id_source[..]).copy_from_slice(&buf[21..25]);
        (&mut msg._snap_shot._trading_phase_code[..]).copy_from_slice(&buf[25..33]);
        
        msg._snap_shot._prev_close_px = byteorder::BigEndian::read_i64(&buf[33..41]);
        msg._snap_shot._num_trades = byteorder::BigEndian::read_i64(&buf[41..49]);
        msg._snap_shot._total_vol_trade = byteorder::BigEndian::read_i64(&buf[49..57]);
        msg._snap_shot._total_value_trade = byteorder::BigEndian::read_i64(&buf[57..65]);

        let entry_count = byteorder::BigEndian::read_u32(&buf[65..69]);
        let mut start : usize = 69;
        for _ in 0..entry_count {
            let mut entry : IndexEntry = Default::default();
            (&mut entry._entry_type[..]).copy_from_slice(&buf[start .. start + 2]);
            entry._entry_px = byteorder::BigEndian::read_i64(&buf[start+ 2 .. start + 10]);
            
            start += 10;
            msg._entries.push(entry);
        }

        info!("Indexsnapshot: {:?}", msg);
        Ok(())
    }

    fn handle_volume_statistic(&self, _:&mut TcpStream, _ : &Vec<u8>)->io::Result<()>{

        Ok(())
    }

    //event dispatcher
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
                self.handle_index_snapshot(stream, buf)?;
            },
            309111=>{
                self.handle_volume_statistic(stream, buf)?;
            },
            _=>{
                info!("{:?}:  {:?}", msg_type, buf);
            }
        };

        Ok(())
    }

    //main run function
    fn run(&mut self) -> io::Result<()> {
        let config = utils::Configuration::load()?;

        let mut stream = TcpStream::connect(&config._addr)?;
        
        self.login(&mut stream)?;
        error!("login success {:?}", stream);

        let mut stream2 = stream.try_clone().unwrap();
        use std::thread;
        thread::Builder::new().name("Heart beat thread".into()).spawn(move || {
            loop {
                use std::time;
                thread::sleep(time::Duration::from_secs(15));
                if let Err(e) = heartbeat(&mut stream2) {
                    error!("heartbeat failed {:?}", e);
                    break;
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

fn main2() {
    
    let mut ctx = Context::new();

    if let Err(e) = ctx.run() {
        error!("{:?}", e);
    }
}

fn main() {
    utils::SimpleLog::init();
    loop {

        main2();
        std::thread::sleep_ms(1000 * 300);
    }
}
