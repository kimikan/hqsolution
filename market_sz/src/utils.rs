
//use std::fs::OpenOptions;

use log;
use log::{LogRecord, LogLevel, LogMetadata, LogLevelFilter};

use serde_json;
use serde::{ Deserialize};
use std::io;
use std::fs::OpenOptions;
use chrono;
use chrono::{Timelike, Datelike};
use std::mem;
use std::slice;

pub fn get_line_number(time : u32)->u32 {
    let h = time / 10000;
    let m = time / 100;
    let s = m % 100;

    let mut pos : u32 = 0;
    if  m >= 1300  && m <= 1500 {
        pos = 121 + (h - 13) * 60 + s;
    } else if m <= 1130 && m >= 930 {
        pos = (h - 9) * 60 + s - 30;
    } else if m < 930 && m >= 925 {
        pos = 0;
    } else if m > 1500 {
        pos = 241;
    } else if m > 1130 && m < 1300 {
        pos = 120;
    } else if  m <= 859 {
        pos = 241;
    }

    return pos;
}

pub fn trading_phase_to_u32(code : &[u8])->u32 {
    if code.len() <= 0 {
        return 0;
    }

    let mut status : u32 = {
        match code[0] {
            b'S' => 1,
            b'O'=>2,
            b'T'=>4,
            b'B'=>5,
            b'C'=>8,
            b'E'=>6,
            b'H'=>7,
            b'A'=>13,
            b'V'=>14,
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

pub fn div_accurate(value : i64, factor: i32)->u32 {
    if factor <= 0 {
        return 0;
    }

    let v1 = value / factor as i64;

    let mut result = v1;

    match factor {
        10=>{
            if value % 10 >= 5 {
                result += 1;
            }
        }
        100=>{
            if value % 100 >= 50 {
                result += 1;
            }
        }
        1000=>{
            if value % 1000 >= 500 { 
                result += 1;
            }
        }
        _=>{
            println!("error div_accurate..... ");
        }
    }

    return result as u32;
}

pub fn is_dept(code : &String)->bool {
    if code.starts_with("10") || code.starts_with("11")
    || code.starts_with("12") || code.starts_with("13") {
        return true;
    }

    return false;
}

pub fn is_fund(code : &String)->bool {
    if code.starts_with("15") || code.starts_with("16")
    || code.starts_with("17") || code.starts_with("18") {
        return true;
    }

    return false;
}

pub fn any_to_u8_slice<T: Sized>(p: &mut T) -> &mut [u8] {
    unsafe { slice::from_raw_parts_mut((p as *mut T) as *mut u8, mem::size_of::<T>()) }
}

use encoding;
pub fn gb2312_to_string(buf : &[u8])->Option<String> {
    let refs = encoding::all::encodings();
    let (x, _) = encoding::decode(buf, encoding::DecoderTrap::Strict, refs[37]);
    
    if let Ok(s) = x {
        return Some(s);
    }

    None
}

pub fn utf8_to_string(buf : &[u8])->String {
    let s = String::from_utf8_lossy(buf);
    s.into_owned()
}

#[derive(Deserialize, Debug)]
pub struct Configuration {
    pub _addr : String,
    pub _static_files : String,
}

impl Configuration {

    pub fn load()->io::Result<Configuration> {
        let file = OpenOptions::new()
                    .read(true).open("default.cfg")?;
        
        use std::io::BufReader;
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();

        use std::io::Read;
        buf_reader.read_to_string(&mut contents)?;

        let r : serde_json::Result<Configuration> = serde_json::from_str(&contents);
   
        if let Ok(c) = r {
            return Ok(c);
        } else {
            println!("{:?}", r);
        }

        Err(io::Error::from(io::ErrorKind::InvalidData))
    }
}


pub struct SimpleLog ;

impl log::Log for SimpleLog {
    fn enabled(&self, m : &LogMetadata)->bool {
        m.level() <= LogLevel::Info
    }

    fn log(&self, r: &LogRecord) {
        return;
        if self.enabled(r.metadata()) {
            
            let path = self.get_file();
            let file_op = OpenOptions::new()
                .create(true).write(true).append(true)
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
        }).unwrap();
    }

    fn get_file(&self)->String {
        let now = chrono::Local::now();
        format!("{}{}{}{}", now.year(), now.month(), now.day(), now.hour())
    }
}

