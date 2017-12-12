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
extern crate chrono;

extern crate tokio_core;
extern crate futures;
extern crate tiberius;
extern crate xml;

mod utils;
mod dbf;
mod t2sdk;
mod db;
mod xmlhelper;
mod interoper;

use log::*;

use std::io;
use std::io::Read;
use std::collections::HashMap;
use std::net::TcpStream;

use byteorder::ByteOrder;

const SENDER: &str = "F000648Q0011";
const TARGET: &str = "VDE";
const HEARTBEAT_INTERVAL: u32 = 150;
const PASSWORD: &str = "F000648Q0011";
const APPVER: &str = "1.00";

#[derive(Debug, Clone)]
struct Snapshot {
    _orig_time: i64,
    _channel_no: u16,
    _md_stream_id: [u8; 3],
    _security_id: [u8; 8],
    _security_id_source: [u8; 4], //102 shenzhen, 103 hongkong
    _trading_phase_code: [u8; 8],
    _prev_close_px: i64,
    _num_trades: i64,
    _total_vol_trade: i64,
    _total_value_trade: i64,
}

impl Default for Snapshot {
    fn default() -> Snapshot {
        Snapshot {
            _orig_time: 0i64,
            _channel_no: 0u16,
            _md_stream_id: [0; 3],
            _security_id: [0; 8],
            _security_id_source: [0; 4],
            _trading_phase_code: [0; 8],
            _prev_close_px: 0i64,
            _num_trades: 0i64,
            _total_vol_trade: 0i64,
            _total_value_trade: 0i64,
        }
    }
}

#[derive(Debug, Clone)]
struct StockEntry {
    _entry_type: [u8; 2],
    _entry_px: i64,
    _entry_size: i64,
    _price_level: u16,
    _num_of_orders: i64,
    //_no_orders : u32,
    _orders: Vec<i64>,
}

impl Default for StockEntry {
    fn default() -> StockEntry {
        StockEntry {
            _entry_type: [0; 2],
            _entry_px: 0i64,
            _entry_size: 0i64,
            _price_level: 0u16,
            _num_of_orders: 0i64,

            _orders: vec![],
        }
    }
}

#[derive(Debug, Clone)]
struct Stock {
    _snap_shot: Snapshot,

    _entries: Vec<StockEntry>,
}

impl Default for Stock {
    fn default() -> Stock {
        Stock {
            _snap_shot: Default::default(),
            _entries: vec![],
        }
    }
}

#[derive(Debug, Clone)]
struct IndexEntry {
    _entry_type: [u8; 2],
    _entry_px: i64,
}

impl Default for IndexEntry {
    fn default() -> IndexEntry {
        IndexEntry {
            _entry_type: [0; 2],
            _entry_px: 0i64,
        }
    }
}

#[derive(Debug, Clone)]
struct Index {
    _snap_shot: Snapshot,
    _entries: Vec<IndexEntry>,
}

#[derive(Debug, Clone)]
struct VolumeStatic {
    _snap_shot: Snapshot,
    _stock_num: u32,
}

impl Default for Index {
    fn default() -> Index {
        Index {
            _snap_shot: Default::default(),
            _entries: vec![],
        }
    }
}

impl Default for VolumeStatic {
    fn default() -> VolumeStatic {
        VolumeStatic {
            _snap_shot: Default::default(),
            _stock_num: 0,
        }
    }
}

#[derive(Debug, Clone)]
struct MsgHead {
    _msg_type: u32,
    _body_length: u32,
}

use std::io::ErrorKind;
fn generate_checksum(bs: &[u8]) -> u32 {
    let mut sum: u32 = 0;
    for i in 0..bs.len() {
        sum += bs[i] as u32;
    }

    sum
}


#[derive(Debug)]
struct Context {
    _stocks: HashMap<String, t2sdk::StockRecord>,
    _config: utils::Configuration,
    _t2ctx: interoper::T2Context,
    _index_statics: HashMap<String, t2sdk::StockRecord>,
    _today: u32,
    _now:u32,

    _stream : Option<TcpStream>,
}

impl Drop for Context {
    fn drop(&mut self) {
        let now = xmlhelper::get_today_date_time();

        println!("Context disposed: {:?}", now);
        if let Some(ref s) = self._stream {
            if s.shutdown(std::net::Shutdown::Both).is_err() {
                println!("Shutdown error");
            }
        }
    }
}


use std::marker::Sized;
use std::slice;
use std::mem;

//msgtype 3, with a 0 len body
fn heartbeat(stream: &mut TcpStream) -> io::Result<()> {
    let mut msg: [u8; 12] = [0; 12];

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

use std::io::Write;
use std::io::Error;


impl Context {

    fn new() -> Context {
        let cfg = utils::Configuration::load();
        println!("{:?}", cfg);

        let stocks_o = utils::load_stocks(utils::STOCKS);
        let statics_o = utils::load_stocks(utils::STATISTICS);

        Context {
            _stocks: stocks_o.unwrap_or(Default::default()),
            _index_statics : statics_o.unwrap_or(Default::default()),
            _config: cfg.unwrap(),
            _t2ctx: interoper::T2Context::new(),
            _today:0,
            _now:0,
            _stream : None,
        }
    }

    fn login(&self, stream: &mut TcpStream) -> io::Result<()> {
        let msg: [u8; 104] = [0x00, 0x00 /* ........ */, 0x00, 0x01, 0x00, 0x00, 0x00, 0x5c,
                              0x46, 0x30 /* .....\F0 */, 0x30, 0x30, 0x36, 0x34, 0x38, 0x51,
                              0x30, 0x30 /* 00648Q00 */, 0x31, 0x31, 0x00, 0x00, 0x00, 0x00,
                              0x0c, 0x00 /* 11...... */, 0x00, 0x00, 0x56, 0x44, 0x45, 0x00,
                              0xb4, 0x00 /* ..VDE... */, 0x00, 0x00, 0x84, 0xff, 0xff, 0xff,
                              0xb4, 0x00 /* ........ */, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00,
                              0x31, 0x35 /* ......15 */, 0x30, 0x00, 0x46, 0x30, 0x30, 0x30,
                              0x36, 0x34 /* 0.F00064 */, 0x38, 0x51, 0x30, 0x30, 0x31, 0x31,
                              0x00, 0x00 /* 8Q0011.. */, 0x00, 0x00, 0x31, 0x2e, 0x30, 0x30,
                              0x00, 0x00 /* ..1.00.. */, 0x00, 0x00, 0x00, 0x00, 0x29, 0xdb,
                              0xb4, 0x00 /* ....)... */, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00,
                              0x00, 0x00 /* ........ */, 0x00, 0x00, 0x0f, 0x00, 0x00, 0x00,
                              0x00, 0x00 /* ........ */, 0x00, 0x00, 0x00, 0x00, 0x00,
                              0x6a /* .....j */];

        let size = stream.write(&msg[0..104])?;
        if size == 104 {
            return Ok(());
        }

        //byteorder::BigEndian::
        return Err(Error::from(ErrorKind::InvalidData));
    }

    #[allow(dead_code)]
    fn login2(&self, stream: &mut TcpStream) -> io::Result<()> {
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
        let mut msg: [u8; 104] = [0; 104];

        byteorder::BigEndian::write_u32(&mut msg[..], 1);
        byteorder::BigEndian::write_u32(&mut msg[4..], 92); //len

        use std::cmp;
        &(msg[8..(8 + cmp::min(20, SENDER.len()))]).copy_from_slice(SENDER.as_bytes());
        &(msg[28..(28 + cmp::min(20, TARGET.len()))]).copy_from_slice(TARGET.as_bytes());

        byteorder::BigEndian::write_u32(&mut msg[48..], HEARTBEAT_INTERVAL); //heartbeat

        &(msg[52..(52 + cmp::min(20, PASSWORD.len()))]).copy_from_slice(PASSWORD.as_bytes());
        &(msg[68..(68 + cmp::min(20, APPVER.len()))]).copy_from_slice(APPVER.as_bytes());

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
    fn get_message(&self, stream: &mut TcpStream) -> (io::Result<u32>, Option<Vec<u8>>) {
        let mut header: [u8; 8] = [0u8; 8];

        let res = stream.read_exact(&mut header[0..8]);
        if let Err(e) = res {
            return (Err(e), None);
        }

        let msg_type = byteorder::BigEndian::read_u32(&header[..]);
        let body_len = byteorder::BigEndian::read_u32(&header[4..]) as usize;

        if body_len <= 0 {
            return (Ok(msg_type), None);
        }

        let mut vec: Vec<u8> = Vec::with_capacity(body_len);
        unsafe {
            vec.set_len(body_len);
        }

        match stream.read_exact(&mut vec) {
            Ok(_) => {
                let mut checksum: [u8; 4] = [0; 4];
                let checksum_res = stream.read_exact(&mut checksum[..]);
                if let Err(e) = checksum_res {
                    return (Err(e), None);
                }

                return (Ok(msg_type), Some(vec));
            }
            Err(e) => {
                return (Err(e), None);
            }
        };
    }

    fn handle_resent_message(&self, _: &mut TcpStream, _: &Vec<u8>) -> io::Result<()> {

        Ok(())
    }

    fn handle_user_report_message(&mut self, _: &mut TcpStream, _: &Vec<u8>) -> io::Result<()> {

        #[derive(Debug)]
        struct ReportMsg {
            _time: i64,
            _version_code: [u8; 16],
            _user_num: u16,
        }
        let mut msg = ReportMsg {
            _time: 0i64,
            _version_code: [0; 16],
            _user_num: 0u16,
        };

        let time: i64;
        let user_num: u16;
        {
            //local variables
            let buf = utils::any_to_u8_slice_mut(&mut msg);
            time = byteorder::BigEndian::read_i64(&buf[..]);
            user_num = byteorder::BigEndian::read_u16(&buf[24..]);
        }
        msg._time = time;
        msg._user_num = user_num;
        t2sdk::push_market_datetime(&mut self._t2ctx, time)?;
        info!("UserReport: {:?}", msg);
        Ok(())
    }

    fn handle_channel_statistic(&self, _: &mut TcpStream, _: &Vec<u8>) -> io::Result<()> {

        Ok(())
    }

    fn handle_market_status_message(&self, _: &mut TcpStream, _: &Vec<u8>) -> io::Result<()> {

        Ok(())
    }

    fn handle_realtime_status(&mut self, _: &mut TcpStream, buf: &Vec<u8>) -> io::Result<()> {

        #[derive(Debug)]
        struct Switcher {
            _switch_type: u16,
            _switch_status: u16,
        }

        #[derive(Debug)]
        struct RealtimeStatus {
            _time: i64,
            _channel_no: u16,
            _security_id: [u8; 8],
            _security_id_source: [u8; 4], //102 shenzhen, 103 hongkong
            _financial_status: [u8; 8],
            _switchers: Vec<Switcher>,
        }
        let mut msg = RealtimeStatus {
            _time: 0i64,
            _channel_no: 0u16,
            _security_id: [0; 8],
            _security_id_source: [0; 4],
            _financial_status: [0; 8],
            _switchers: vec![],
        };

        msg._time = byteorder::BigEndian::read_i64(&buf[..]);
        msg._channel_no = byteorder::BigEndian::read_u16(&buf[8..]);

        (&mut msg._security_id[..]).copy_from_slice(&buf[10..18]);
        (&mut msg._security_id_source[..]).copy_from_slice(&buf[18..22]);
        (&mut msg._financial_status[..]).copy_from_slice(&buf[22..30]);

        let count = byteorder::BigEndian::read_u32(&buf[30..34]);

        let security_id = utils::utf8_to_string(&msg._security_id[0..6]);
        let stock = self._stocks
            .entry(security_id.clone())
            .or_insert(t2sdk::StockRecord::default());

        for i in 0..count {
            let mut s = Switcher {
                _switch_type: 0u16,
                _switch_status: 0u16,
            };

            s._switch_type = byteorder::BigEndian::read_u16(&buf[(34 + i * 4) as usize..]);
            s._switch_status = byteorder::BigEndian::read_u16(&buf[(34 + i * 4 + 2) as usize..]);

            if s._switch_type == 1 {
                if s._switch_status == 1 {
                    stock._margin_status |= 0x1;
                }
            } else if s._switch_type == 2 {
                if s._switch_status == 1 {
                    stock._margin_status |= 0x2;
                }
            }

            msg._switchers.push(s);
        }

        t2sdk::push_market_datetime(&mut self._t2ctx, msg._time)?;
        info!("Realtime: {:?}", msg);
        Ok(())
    }

    fn handle_stock_report(&mut self, _: &mut TcpStream, buf: &Vec<u8>) -> io::Result<()> {
        #[derive(Debug)]
        struct StockReport {
            _time: i64,
            _channel_no: u16,
            _news_id: [u8; 8],
            _head_line: String, //[u8; 128],
            _raw_data_format: [u8; 8], //txt, pdf, doc
            _raw_data_length: u32,
            _raw_data: Vec<u8>,
        }
        let mut msg = StockReport {
            _time: 0i64,
            _channel_no: 0u16,
            _news_id: [0; 8],
            _head_line: "".to_owned(), //[0; 128],
            _raw_data_format: [0; 8],
            _raw_data_length: 0u32,
            _raw_data: vec![],
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
            msg._raw_data
                .reserve_exact(msg._raw_data_length as usize);
            unsafe {
                msg._raw_data.set_len(msg._raw_data_length as usize);
            }
            (&mut msg._raw_data[..]).copy_from_slice(&buf[158..]);
        }

        t2sdk::push_market_datetime(&mut self._t2ctx, msg._time)?;
        info!("Stockreport: {:?}", msg);
        Ok(())
    }

    fn parse_entry(stock: &mut t2sdk::StockRecord, entry: &StockEntry) {
        match entry._entry_type[0] {
            b'0' => {
                //buying
                let index = (entry._price_level - 1) as usize;
                if index < 5 {
                    stock._buy_amounts[index] = utils::div_accurate(entry._entry_size, 100);
                    stock._buy_pxs[index] = utils::div_accurate(entry._entry_px, 1000);
                }
            }
            b'1' => {
                //selling
                let index = (entry._price_level - 1) as usize;
                if index < 5 {
                    stock._sale_amounts[index] = utils::div_accurate(entry._entry_size, 100);
                    stock._sale_pxs[index] = utils::div_accurate(entry._entry_px, 1000);
                }
            }
            b'2' | b'3' => {
                stock._last_px = utils::div_accurate(entry._entry_px, 1000);

                if stock._total_shares > 0 && stock._last_px > 0 {
                    stock._market_value = (stock._total_shares * (stock._last_px as i64) / 1000) as u64;
                }

                if stock._static_pe_rate > 0 && stock._last_px > 0 {
                    stock._pe_rate = (stock._last_px * 100) / stock._static_pe_rate;
                }

                if stock._static_dynamic_pe > 0 && stock._last_px > 0 {
                    stock._dynamic_pe = (stock._last_px * 100) / stock._static_dynamic_pe;
                }
            }
            b'4' => {
                stock._open_px = utils::div_accurate(entry._entry_px, 1000);
            }
            b'7' => {
                stock._high_px = utils::div_accurate(entry._entry_px, 1000);
            }
            b'8' => {
                stock._low_px = utils::div_accurate(entry._entry_px, 1000);
            }
            b'x' => {
                match entry._entry_type[1] {
                    /*b'5'=>{
                        stock._pe_rate = entry._entry_px / 10000 as u32;
                    }
                    b'6'=>{
                        stock._dynamic_pe = entry._entry_px / 10000 as u32;
                    } */
                    b'7' => {
                        //fund value
                        if entry._entry_px > 0 {
                            stock._market_value = entry._entry_px as u64;
                        }
                    }
                    b'a' => {
                         //index
                        stock._pre_close_px = utils::div_accurate(entry._entry_px, 1000);
                    }
                    b'b' => {
                        stock._open_px = utils::div_accurate(entry._entry_px, 1000);
                    }
                    b'c' => {
                        stock._high_px = utils::div_accurate(entry._entry_px, 1000);
                    }
                    b'd' => {
                        stock._low_px = utils::div_accurate(entry._entry_px, 1000);
                    }
                    b'e' => {
                        //stock._up_limit
                    }
                    b'f' => {
                        //down limit
                    }
                    _ => {} //end second byte
                }
            }
            _ => {}//end first byte
        };
    }

    //the main function is this one
    fn handle_stock_snapshot(&mut self, _: &mut TcpStream, buf: &Vec<u8>) -> io::Result<()> {

        let mut msg: Stock = Default::default();
        //msg._snap_shot._orig_time = byteorder::BigEndian::readi64()
        msg._snap_shot._orig_time = byteorder::BigEndian::read_i64(&buf[..]);

        t2sdk::push_market_datetime(&mut self._t2ctx, msg._snap_shot._orig_time)?;

        msg._snap_shot._channel_no = byteorder::BigEndian::read_u16(&buf[8..]);
        (&mut msg._snap_shot._md_stream_id[..]).copy_from_slice(&buf[10..13]);
        (&mut msg._snap_shot._security_id[..]).copy_from_slice(&buf[13..21]);
        let security_id = utils::utf8_to_string(&msg._snap_shot._security_id[0..6]);

        let stock = self._stocks
            .entry(security_id.clone())
            .or_insert(t2sdk::StockRecord::default());

        stock._date = (msg._snap_shot._orig_time / 1000000000) as u32;
        stock._time = ((msg._snap_shot._orig_time / 1000) % 1000000) as u32;
        stock._stock_code = security_id.clone();

        (&mut msg._snap_shot._security_id_source[..]).copy_from_slice(&buf[21..25]);
        (&mut msg._snap_shot._trading_phase_code[..]).copy_from_slice(&buf[25..33]);
        stock._trade_status = utils::trading_phase_to_u32(&msg._snap_shot._trading_phase_code[..]);
        stock._line_no = utils::get_line_number(stock._time);

        msg._snap_shot._prev_close_px = byteorder::BigEndian::read_i64(&buf[33..41]);
        stock._pre_close_px = (msg._snap_shot._prev_close_px / 10) as u32;

        msg._snap_shot._num_trades = byteorder::BigEndian::read_i64(&buf[41..49]);

        msg._snap_shot._total_vol_trade = byteorder::BigEndian::read_i64(&buf[49..57]);
        stock._trade_amount = (msg._snap_shot._total_vol_trade / 100);
        //println!("code:{}, shares: {:?}, amount: {}",stock._stock_code,  stock._nonstrict_shares, stock._trade_amount);
        if stock._nonstrict_shares > 0 {
            if stock._trade_amount > 0 {
                stock._change_rate = ((stock._trade_amount * 100000/ stock._nonstrict_shares as i64 + 5 ) / 10) as u32;
            }
        }

        msg._snap_shot._total_value_trade = byteorder::BigEndian::read_i64(&buf[57..65]);
        stock._trade_balance = (msg._snap_shot._total_value_trade / 10);

        let entry_count = byteorder::BigEndian::read_u32(&buf[65..69]);
        let mut start: usize = 69;
        for _ in 0..entry_count {
            let mut entry: StockEntry = Default::default();

            (&mut entry._entry_type[..]).copy_from_slice(&buf[start..start + 2]);

            entry._entry_px = byteorder::BigEndian::read_i64(&buf[start + 2..start + 10]);
            entry._entry_size = byteorder::BigEndian::read_i64(&buf[start + 10..start + 18]);
            entry._price_level = byteorder::BigEndian::read_u16(&buf[start + 18..start + 20]);
            entry._num_of_orders = byteorder::BigEndian::read_i64(&buf[start + 20..start + 28]);

            let qty_count = byteorder::BigEndian::read_u32(&buf[start + 28..start + 32]);

            Context::parse_entry(stock, &entry);

            let mut start2 = start + 32;
            for _ in 0..qty_count {

                let qty = byteorder::BigEndian::read_i64(&buf[start2..start2 + 8]);
                entry._orders.push(qty);
                start2 += 8;
            }

            start = start2;
            msg._entries.push(entry);
        }

        if utils::is_dept(&security_id) {
            stock._stock_type = 3;
            if stock._time >= 91500 {
                t2sdk::push_debt(&mut self._t2ctx, stock)?;
            }
        } else if utils::is_fund(&security_id) {
            stock._stock_type = 4;

            if stock._time >= 91500 {
                t2sdk::push_fund(&mut self._t2ctx, stock)?;
            }
        } else {
            stock._stock_type = 1;

            if stock._time >= 91500 {
                t2sdk::push_stock(&mut self._t2ctx, stock)?;
            }
        }

        info!("Stocksnapshot: {:?}", msg);
        Ok(())
    }

    fn handle_index_snapshot(&mut self, _: &mut TcpStream, buf: &Vec<u8>) -> io::Result<()> {

        let mut msg: Index = Default::default();
        //msg._snap_shot._orig_time = byteorder::BigEndian::readi64()
        msg._snap_shot._orig_time = byteorder::BigEndian::read_i64(&buf[..]);
        msg._snap_shot._channel_no = byteorder::BigEndian::read_u16(&buf[8..]);
        t2sdk::push_market_datetime(&mut self._t2ctx, msg._snap_shot._orig_time)?;

        (&mut msg._snap_shot._md_stream_id[..]).copy_from_slice(&buf[10..13]);
        (&mut msg._snap_shot._security_id[..]).copy_from_slice(&buf[13..21]);
        (&mut msg._snap_shot._security_id_source[..]).copy_from_slice(&buf[21..25]);
        (&mut msg._snap_shot._trading_phase_code[..]).copy_from_slice(&buf[25..33]);

        let security_id = utils::utf8_to_string(&msg._snap_shot._security_id[0..6]);

        let stock = self._stocks
            .entry(security_id.clone())
            .or_insert(t2sdk::StockRecord::default());

        stock._date = (msg._snap_shot._orig_time / 1000000000) as u32;
        stock._time = ((msg._snap_shot._orig_time / 1000) % 1000000) as u32;
        stock._stock_code = security_id.clone();

        msg._snap_shot._prev_close_px = byteorder::BigEndian::read_i64(&buf[33..41]);
        msg._snap_shot._num_trades = byteorder::BigEndian::read_i64(&buf[41..49]);
        msg._snap_shot._total_vol_trade = byteorder::BigEndian::read_i64(&buf[49..57]);
        msg._snap_shot._total_value_trade = byteorder::BigEndian::read_i64(&buf[57..65]);

        stock._trade_status = utils::trading_phase_to_u32(&msg._snap_shot._trading_phase_code[..]);
        stock._line_no = utils::get_line_number(stock._time);
        stock._pre_close_px = (msg._snap_shot._prev_close_px / 10) as u32;

        let key = utils::translate(&security_id);

        stock._trade_amount = msg._snap_shot._total_vol_trade / 100;
        stock._trade_balance = (msg._snap_shot._total_value_trade / 10);

        if let Some(k) = key {
            let value = self._index_statics.get(&k);
            if let Some(v) = value {
                stock._trade_amount = v._trade_amount;
                stock._trade_balance = v._trade_balance;
            }
        }
        
        let entry_count = byteorder::BigEndian::read_u32(&buf[65..69]);
        let mut start: usize = 69;
        for _ in 0..entry_count {
            let mut entry: IndexEntry = Default::default();
            (&mut entry._entry_type[..]).copy_from_slice(&buf[start..start + 2]);
            entry._entry_px = byteorder::BigEndian::read_i64(&buf[start + 2..start + 10]);

            let mut stock_entry = StockEntry::default();
            stock_entry._entry_type = entry._entry_type;
            stock_entry._entry_px = entry._entry_px;

            Context::parse_entry(stock, &stock_entry);
            start += 10;
            msg._entries.push(entry);
        }

        stock._stock_type = 2;

        if stock._time >= 91500 {
            t2sdk::push_index(&mut self._t2ctx, stock)?;
        }
        info!("Indexsnapshot: {:?}", msg);
        Ok(())
    }

    fn handle_volume_statistic(&mut self, _: &mut TcpStream, buf: &Vec<u8>) -> io::Result<()> {

        let mut msg: VolumeStatic = Default::default();
        //msg._snap_shot._orig_time = byteorder::BigEndian::readi64()
        msg._snap_shot._orig_time = byteorder::BigEndian::read_i64(&buf[..]);
        msg._snap_shot._channel_no = byteorder::BigEndian::read_u16(&buf[8..]);
        t2sdk::push_market_datetime(&mut self._t2ctx, msg._snap_shot._orig_time)?;

        (&mut msg._snap_shot._md_stream_id[..]).copy_from_slice(&buf[10..13]);
        (&mut msg._snap_shot._security_id[..]).copy_from_slice(&buf[13..21]);
        (&mut msg._snap_shot._security_id_source[..]).copy_from_slice(&buf[21..25]);
        (&mut msg._snap_shot._trading_phase_code[..]).copy_from_slice(&buf[25..33]);

        let security_id = utils::utf8_to_string(&msg._snap_shot._security_id[0..6]);

        let stock = self._index_statics
            .entry(security_id.clone())
            .or_insert(t2sdk::StockRecord::default());

        stock._date = (msg._snap_shot._orig_time / 1000000000) as u32;
        stock._time = ((msg._snap_shot._orig_time / 1000) % 1000000) as u32;
        stock._stock_code = security_id;

        msg._snap_shot._prev_close_px = byteorder::BigEndian::read_i64(&buf[33..41]);
        msg._snap_shot._num_trades = byteorder::BigEndian::read_i64(&buf[41..49]);
        msg._snap_shot._total_vol_trade = byteorder::BigEndian::read_i64(&buf[49..57]);
        msg._snap_shot._total_value_trade = byteorder::BigEndian::read_i64(&buf[57..65]);

        stock._trade_status = utils::trading_phase_to_u32(&msg._snap_shot._trading_phase_code[..]);
        stock._line_no = utils::get_line_number(stock._time);
        stock._pre_close_px = (msg._snap_shot._prev_close_px / 10) as u32;
        stock._trade_amount = msg._snap_shot._total_vol_trade / 10000;
        stock._trade_balance = (msg._snap_shot._total_value_trade / 10);

        let stock_num = byteorder::BigEndian::read_u32(&buf[65..69]);
        //stock._stock_num = stock_num;

        Ok(())
    }

    //event dispatcher
    fn handle_message(&mut self,
                      stream: &mut TcpStream,
                      msg_type: u32,
                      buf: &Vec<u8>)
                      -> io::Result<()> {

        match msg_type {
            //heartbeat
            3 => {
                heartbeat(stream)?;
            }
            //channel heartbeat
            390095 => {
                //channel heartbeat
            }
            390094 => {
                self.handle_resent_message(stream, buf)?;
            }
            390093 => {
                self.handle_user_report_message(stream, buf)?;
            }
            390090 => {
                self.handle_channel_statistic(stream, buf)?;
            }
            //market status
            390019 => {
                self.handle_market_status_message(stream, buf)?;
            }
            //realtime status
            390013 => {
                self.handle_realtime_status(stream, buf)?;
            }
            390012 => {
                self.handle_stock_report(stream, buf)?;
            }
            300111 => {
                self.handle_stock_snapshot(stream, buf)?;
            }
            300611 => {
                //hongkong stocks
            }
            309011 => {
                self.handle_index_snapshot(stream, buf)?;
            }
            309111 => {
                self.handle_volume_statistic(stream, buf)?;
            }
            _ => {
                info!("{:?}:  {:?}", msg_type, buf);
            }
        };

        Ok(())
    }

    //main run function
    fn run(&mut self) -> io::Result<()> {

        let mut stream : TcpStream = TcpStream::connect(&self._config._addr)?;
        self._stream = Some(stream.try_clone().unwrap());

        self.login(&mut stream)?;
        error!("login success {:?}", stream);

        let mut stream2 = stream.try_clone().unwrap();
        use std::thread;
        thread::Builder::new()
            .name("Heart beat thread".into())
            .spawn(move || {
                loop {
                    use std::time;
                    thread::sleep(time::Duration::from_secs(15));
                    if let Err(e) = heartbeat(&mut stream2) {
                        error!("heartbeat failed {:?}", e);
                        break;
                    }
                }
                println!("heartbeat failed!");

            })
            .unwrap();

        loop {
            match utils::check_time() {
                Ok(time)=>{
                    if time >= 840 && time <= 905 {
                        //parse static files
                        if let Err(e) = self.prepare_market_init() {
                            println!("Market init failed: {:?}", e);
                        }
                    }
                }
                Err(e)=>{
                    return Err(e);
                }
            };

            if let Err(e) = utils::save_to_disk(&self._stocks, &self._index_statics) {
                println!("Save to snapshot: {:?}", e);
            }

            let (res, op) = self.get_message(&mut stream);
            let msg_type = match res {
                Ok(expr) => expr,
                Err(e) => {
                    return Err(e);
                }
            };

            if let Some(buf) = op {
                let _ = self.handle_message(&mut stream, msg_type, &buf)?;
            }
        }
        //Ok(())
    }

    fn prepare_market_init(&mut self)->io::Result<()>{
        //println!("fff");
        use xmlhelper;
        let (date, time) = xmlhelper::get_today_date_time();

        if self._today == date {

            if self._now <= 85000 && time > 85000 {
                xmlhelper::parse_static_files(self._config._static_files.as_str(),
                                                            &mut self._stocks,
                                                            date)?;
                println!("Parse static files success!");

                self.init()?;
                println!("Sqlserver sync success!");
            }

            if self._now < 90000 && time >= 90000 {
                for (_, value) in &self._stocks {

                    //2:index, 3:Fund, 4:Debt, 1:Stock
                    if value._stock_type == 1 {
                        t2sdk::push_stock(&mut self._t2ctx, &value)?;
                    } else if value._stock_type == 2 {
                        t2sdk::push_index(&mut self._t2ctx, &value)?;
                    } else if value._stock_type == 3 {
                        t2sdk::push_fund(&mut self._t2ctx, &value)?;
                    } else if value._stock_type == 4 {
                        t2sdk::push_debt(&mut self._t2ctx, &value)?;
                    }
                } //end for
                //initialize
                t2sdk::push_market_status(&mut self._t2ctx, date, time, 2)?;
                println!("Sent market initialize event success!");
            }
        }

        self._today = date;
        self._now = time;

        Ok(())
        
    }
    
    fn init(&mut self) -> io::Result<()> {

        let server_o = db::Sqlserver::new();
        if let Some(mut s) = server_o {
            s.update(&mut self._stocks)?;
            s.update2(&mut self._stocks)?;
            
            return Ok(());
        }

        return Err(io::Error::from(io::ErrorKind::InvalidData));
    }
}

fn main2() {
    //println!("ee");
    let mut ctx = Context::new();

    //no need?  better have, whatever
    if let Err(e) = ctx.init() {
        error!("{:?}", e);
        println!("{:?}", e);
        return;
    }

    //println!("{:?}", ctx._stocks);
    if let Err(e) = ctx.run() {
        error!("run failed: {:?}", e);
        println!("run failed: {:?}", e);
    }
}

fn test_send() {
    let mut ctx = interoper::T2Context::new();
    //ctx.set_callback(interoper::callback);
    println!("xxxxxx");
    t2sdk::push_market_time(&mut ctx, 0, 1).unwrap();

    std::thread::sleep_ms(1000 * 5);
}

fn test_save() ->io::Result<()>{
    let mut ctx = Context::new();
    use xmlhelper;
    let (date, _) = xmlhelper::get_today_date_time();
    //println!("xxc");
    xmlhelper::parse_static_files(ctx._config._static_files.as_str(),
                                                  &mut ctx._stocks,
                                                  date)?;
    ctx.init()?;

    use serde_json;
    use std::io::BufWriter;
    use std::fs::OpenOptions;
    let file = OpenOptions::new().write(true).create(true)
                .open("data")?;
    let mut buf_wr = BufWriter::new(file);
    //let mut contents = String::new();

    use std::io::Write;
    let j = serde_json::to_string(&ctx._stocks)?;

    buf_wr.write_all(j.as_bytes())?;

    Ok(())
}

fn main() {
    /*
    if let Err(e) = test_save() {
        println!("{:?}", e);
    }

    return; */
    /*
    for _ in 0..20 {
        let s = "002255".to_owned();
        let s2 = "002245".to_owned();
        //let x = interoper::to_char_array(&s);

        use std::ffi::CString;
        //unsafe { interoper::test(CString::new(s.as_str()).unwrap().as_ptr()); }
        unsafe { interoper::test(CString::new(s.as_str()).unwrap().as_ptr(), CString::new(s2.as_str()).unwrap().as_ptr()); }
        println!("{:?}", s);
    }
    return; */
    utils::SimpleLog::init();
    loop {

        main2();

        if let Err(_) = utils::check_time() {
            std::thread::sleep_ms(1000 * 300);
        } else {
            std::thread::sleep_ms(1000 * 1);
        }
    }
}
