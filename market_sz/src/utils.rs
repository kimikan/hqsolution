
//use std::fs::OpenOptions;

use log;
use log::{LogRecord, LogLevel, LogMetadata, LogLevelFilter};

use serde_json;
use serde::Deserialize;
use std::io;
use std::fs::OpenOptions;
use chrono;
use chrono::{Timelike, Datelike};
use std::mem;
use std::slice;

pub fn get_line_number(time: u32) -> u32 {
    let h = time / 10000;
    let m = time / 100;
    let s = m % 100;

    let mut pos: u32 = 0;
    if m >= 1300 && m <= 1500 {
        pos = 121 + (h - 13) * 60 + s;
    } else if m <= 1130 && m >= 930 {
        pos = (h - 9) * 60 + s - 30;
    } else if m < 930 && m >= 925 {
        pos = 0;
    } else if m > 1500 {
        pos = 241;
    } else if m > 1130 && m < 1300 {
        pos = 120;
    } else if m <= 859 {
        pos = 241;
    }

    return pos;
}

pub fn trading_phase_to_u32(code: &[u8]) -> u32 {
    if code.len() <= 0 {
        return 0;
    }

    let mut status: u32 = {
        match code[0] {
            b'S' => 1,
            b'O' => 2,
            b'T' => 4,
            b'B' => 5,
            b'C' => 8,
            b'E' => 6,
            b'H' => 7,
            b'A' => 13,
            b'V' => 14,
            _ => 0,
        }
    };

    if code.len() >= 2 {
        if code[1] == b'1' {
            status = 9;
        }
    }

    status
}

pub fn div_accurate(value: i64, factor: i32) -> u32 {
    if factor <= 0 {
        return 0;
    }

    let v1 = value / factor as i64;

    let mut result = v1;

    match factor {
        10 => {
            if value % 10 >= 5 {
                result += 1;
            }
        }
        100 => {
            if value % 100 >= 50 {
                result += 1;
            }
        }
        1000 => {
            if value % 1000 >= 500 {
                result += 1;
            }
        }
        _ => {
            println!("error div_accurate..... ");
        }
    }

    return result as u32;
}

pub fn is_dept(code: &String) -> bool {
    if code.starts_with("10") || code.starts_with("11") || code.starts_with("12") ||
       code.starts_with("13") {
        return true;
    }

    return false;
}

pub fn is_fund(code: &String) -> bool {
    if code.starts_with("15") || code.starts_with("16") || code.starts_with("17") ||
       code.starts_with("18") {
        return true;
    }

    return false;
}

//check time...
pub fn check_time() -> io::Result<u32> {
    use chrono::Timelike;
    let now = chrono::Local::now();
    let hour = now.hour();
    let time = hour * 100 + now.minute();

    //0 based, monday from
    let weekday = now.weekday().num_days_from_monday();
    if (time >= 1530 || time <= 830) || weekday >= 5 {

        return Err(io::Error::from(io::ErrorKind::WouldBlock));
    }

    Ok(time)
}

pub fn translate(code : &String)->Option<String> {
    if code.eq("399001") {
        return Some("395099".to_owned());
    }

    if code.eq("399002") {
        return Some("395001".to_owned());
    }

    if code.eq("399003") {
        return Some("395002".to_owned());
    }

    if code.eq("399005") {
        return Some("395003".to_owned());
    }

    if code.eq("399006") {
        return Some("395004".to_owned());
    }
    
    None
}

pub fn any_to_u8_slice_mut<T: Sized>(p: &mut T) -> &mut [u8] {
    unsafe { slice::from_raw_parts_mut((p as *mut T) as *mut u8, mem::size_of::<T>()) }
}

use encoding;
pub fn gb2312_to_string(buf: &[u8]) -> Option<String> {
    let refs = encoding::all::encodings();
    let (x, _) = encoding::decode(buf, encoding::DecoderTrap::Strict, refs[37]);

    if let Ok(s) = x {
        return Some(s);
    }

    None
}

use encoding::{Encoding, ByteWriter, EncoderTrap, DecoderTrap};
use encoding::types::RawEncoder;
use encoding::all::GBK;

fn hex_ncr_escape(_encoder: &mut RawEncoder, input: &str, output: &mut ByteWriter) -> bool {
    let escapes: Vec<String> = input
        .chars()
        .map(|ch| format!("&#x{:x};", ch as isize))
        .collect();
    let escapes = escapes.concat();
    output.write_bytes(escapes.as_bytes());
    true
}

static HEX_NCR_ESCAPE: EncoderTrap = EncoderTrap::Call(hex_ncr_escape);
pub fn string_to_gb2312(s: &String) -> Vec<u8> {
    let x = GBK.encode(s, HEX_NCR_ESCAPE);

    let v: Vec<u8> = vec![];
    x.unwrap_or(v)
}

pub fn utf8_to_string(buf: &[u8]) -> String {
    let s = String::from_utf8_lossy(buf);
    s.into_owned()
}

#[derive(Deserialize, Debug, Clone)]
pub struct Configuration {
    pub _addr: String,
    pub _static_files: String,
}

impl Configuration {
    pub fn load() -> io::Result<Configuration> {
        let file = OpenOptions::new().read(true).open("default.cfg")?;

        use std::io::BufReader;
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();

        use std::io::Read;
        buf_reader.read_to_string(&mut contents)?;

        let r: serde_json::Result<Configuration> = serde_json::from_str(&contents);

        if let Ok(c) = r {
            return Ok(c);
        } else {
            println!("{:?}", r);
        }

        Err(io::Error::from(io::ErrorKind::InvalidData))
    }
}


pub struct SimpleLog;

impl log::Log for SimpleLog {
    fn enabled(&self, m: &LogMetadata) -> bool {
        m.level() <= LogLevel::Info
    }

    fn log(&self, r: &LogRecord) {

        return;
        if self.enabled(r.metadata()) {

            let path = self.get_file();
            let file_op = OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&path);

            if let Ok(mut f) = file_op {
                use std::io::Write;

                //println!("{:?}", r.args());
                if let Err(e) = writeln!(f, "{:?}", r.args()) {
                    println!("{:?}", e);
                } 
            }
        }
    }
}

impl SimpleLog {
    pub fn init() {
        log::set_logger(|max_log_level| {
                            max_log_level.set(LogLevelFilter::Info);

                            Box::new(SimpleLog)
                        })
                .unwrap();
    }

    fn get_file(&self) -> String {
        //return "test".to_owned();
        let now = chrono::Local::now();
        format!("{}{}{}{}", now.year(), now.month(), now.day(), now.hour())
    }
}

use std::collections::HashMap;
pub fn save_stocks(stocks : &HashMap<String, t2sdk::StockRecord>, file_name:&str)->io::Result<()>{

    use serde_json;
    use std::io::BufWriter;
    use std::fs::OpenOptions;
    let file = OpenOptions::new().write(true).create(true)
                .open(file_name)?;
    let mut buf_wr = BufWriter::new(file);

    use std::io::Write;
    let j = serde_json::to_string(stocks)?;

    buf_wr.write_all(j.as_bytes())?;
    Ok(())
}


pub const STOCKS:&str = "stocks.json";
pub const STATISTICS:&str = "statistics.json";

use t2sdk;
pub fn load_stocks(file_name:&str)->Option<HashMap<String, t2sdk::StockRecord>> {

    use serde_json;
    use std::io::BufReader;
    use std::fs::OpenOptions;
    let file_r = OpenOptions::new().read(true)
                .open(file_name);
    
    if let Ok(file) = file_r {
        let mut buf_r = BufReader::new(file);
        use std::io::Read;
        use serde_json::error;
        let j_r : error::Result<HashMap<String, t2sdk::StockRecord>> = serde_json::from_reader(buf_r);

        if let Ok(j) = j_r {
            println!("Load success: {:?}", file_name);
            return Some(j);
        }
    }
    
    None
}

static mut __SAVE_TIME:u32  = 0;
pub fn save_to_disk(stocks:&HashMap<String, t2sdk::StockRecord>
    , statics:&HashMap<String, t2sdk::StockRecord>)->io::Result<()>{

    let mut need_to_save : bool = false;

    use chrono::Timelike;
    let now = chrono::Local::now();
    let hour = now.hour();
    let time = hour * 100 + now.minute();
    let time2 = hour * 60 + now.minute();
    if time >= 900 && time <= 1600 {
        unsafe {
            if time2 > __SAVE_TIME {
                if time2 >= __SAVE_TIME + 15 {
                    need_to_save = true;
                    __SAVE_TIME = time2;
                }
            } else {
                __SAVE_TIME = time2;
            }
        }
    }

    if need_to_save {
        println!("Save to disk {}", time);
        save_stocks(&stocks, STOCKS)?;
        save_stocks(&statics, STATISTICS)?;
    }

    Ok(())
}