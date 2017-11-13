
//use std::fs::OpenOptions;
<<<<<<< HEAD

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
                .create(true).write(true).truncate(true)
                .open(path);
            
            if let Ok(mut f) = file_op {
                use std::io::Write;
                
                if let Err(e) = writeln!(f, "{:?}", r) {
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
=======
>>>>>>> f57edc8fb2f4e8ab2acdb3ab4c97f086d872d37b

