
//use std::fs::OpenOptions;

use log;
use log::{LogRecord, LogLevel, LogMetadata, LogLevelFilter};

pub struct SimpleLog ;

impl log::Log for SimpleLog {
    fn enabled(&self, m : &LogMetadata)->bool {
        m.level() <= LogLevel::Info
    }

    fn log(&self, r: &LogRecord) {
        if self.enabled(r.metadata()) {
            use std::fs::OpenOptions;
            let path = "default.log";
            let file_op = OpenOptions::new()
                .create(true).write(true).append(true)
                .open(path);
            
            if let Ok(mut f) = file_op {
                use std::io::Write;
                
				println!("{:?}", r.args());
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
}

