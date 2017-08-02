
extern crate encoding;

mod utils;

use std::io;
use std::fs::{OpenOptions, File};
use std::io::{BufReader, BufRead, Read};

use std::collections::HashMap;

struct Context {
    _prev_len:usize,
    _stocks:HashMap<String, [u8]>,
}

impl Context {
    fn new()->Context {
        Context{
            _prev_len:0,
            _stocks:Default::default(),
        }
    }
}

trait LineReader<R:Read> {
     fn get_line(&mut self, buf: &mut [u8]) -> io::Result<usize>;
     //fn get_line(&mut self) -> io::Result<usize>;
}

impl<R:Read> LineReader<R> for BufReader<R> {
    /*fn get_line(&mut self) -> Result<usize> {
        Ok(())
     } */
    
     fn get_line(&mut self, buf: &mut [u8]) -> io::Result<usize> {
         let mut index = 0usize;
        loop {
            let size = self.read(&mut buf[index..index+1])?;
            if size > 0 {
                if b'\n' == buf[index] {
                    return Ok(index + 1);
                }
                index += 1;
            } else {
                return Ok(index);
            }
        }

        use std::io::{Error, ErrorKind};
        Err(Error::from(ErrorKind::Interrupted))
     }
}

//process record ......
fn process_record(ctx:&mut Context, line:&[u8])->io::Result<()> {
    let stream_id = &line[..5];
    let security_id = &line[6..12];
    let name = &line[13..21];
    let id = String::from_utf8_lossy(stream_id);
    let code = String::from_utf8_lossy(security_id);

    let refs = encoding::all::encodings();

    use encoding::DecoderTrap;
    let (name2, _) = encoding::decode(name, DecoderTrap::Strict, refs[37]);
    println!("{:?}, {:?} {:?}", id, code, name2);
    //println!("{:?}, {:?}", String::from_utf8(stream_id).unwrap(), 
    //encoding::all::encoding::decode(security_id, DecoderTrap::Strict, refs[37]));
    Ok(())
}

//parse header indicates that, if any needs to be updated
fn process_header(ctx:&mut Context, reader:&mut BufReader<File>)->io::Result<bool>{
    let mut str:String = String::new();
    let size = reader.read_line(&mut str)?;
    println!("{:?}, {:?}", size, str);

    if size > 0 && str.len() > 26 {
        let file_len = &str[16..26];

        let len = file_len.trim().parse::<usize>();
        if let Ok(l) = len {
            if l != ctx._prev_len {
                println!("file len: {}", l);
                ctx._prev_len = l;
                return Ok(true);
            }
        } else {
            println!("{:?}", len);
        }//end let
    }

    Ok(false)
}

//handle the shenzhen txt file line by line
//bool, true = successfully handled & has changed stocks
//false means,   no changes, no errors
fn process_file(mut ctx:Context, file:&str)->io::Result<bool> {
    let file = OpenOptions::new()
        .read(true)
        .open(file)?;

    let mut reader:BufReader<File> = BufReader::new(file);
    {
        let handle_more = process_header(&mut ctx, &mut reader)?;

        //the bool value means if has any changes 
        //in the records
        if !handle_more {
            return Ok(false);
        }
    }

    let mut vec:Vec<u8> = Vec::with_capacity(1024);
    unsafe{ vec.set_len(1024); }
    loop {
        let size = reader.get_line(&mut vec)?;
        
        if size == 0 {
            println!("size: {:?}", size);
            break;
        }

        if size < 100 {
            continue;
        }

        if let Err(e) = process_record(&mut ctx, &vec[..size]) {
            return Err(e);
        }
    }

    Ok(true)
}

//the main function entry point
fn main() {
    let mut ctx = Context::new();
    if let Err(result) = process_file(ctx, "MKTDT00.TXT") {
        println!("{}", result);
    }
}
