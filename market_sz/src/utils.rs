
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
#[derive(Deserialize, Debug)]
pub struct Configuration {
    pub _addr : String,
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

