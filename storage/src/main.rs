extern crate leveldb;

use leveldb::database::Database;
use leveldb::kv::KV;
use leveldb::options::{Options,WriteOptions,ReadOptions};
use leveldb::iterator::Iterable;

use std::path::Path;

#[repr(C, packed)]
struct KLine {

  _balance:u64,
  _amount:u64,
  _radio:u32,
  _data_time:u32,

  _open_price:u32,
  _close_price:u32,
  _high_price:u32,
  _low_price:u32,
}

impl Default for KLine {
  fn default() ->KLine {
    KLine {
      _amount:0,
      _balance:0,
      _radio:0,
      _data_time:0,
      _open_price:0,
      _close_price:0,
      _high_price:0,
      _low_price:0,
    }
  }
}


impl KLine {
  //transfer struct to u8 slice
  fn as_bytes(&self) -> &[u8] {
    use std::slice;
    use std::mem;

    unsafe {
      slice::from_raw_parts((self as * const KLine) as * const u8, mem::size_of::<KLine>())
    }
  }
}

#[derive(Clone, Debug)]
struct Uri {
  _market:u32,
  _code:String,
  _line_no:u32,
}

impl Default for Uri {
  fn default()->Uri {
    Uri {
      _market:0,
      _code:Default::default(),
      _line_no:0,
    }
  }
}

impl<'a> From<&'a [u8]> for Uri {
  fn from(buf:&[u8])->Uri {
    panic!("xxxx")
    //Uri::default()
  }
}

use std::mem;
use std::slice;

fn to_bytes(value:&u32)->&[u8] {
  let v: *const u32 = value as *const u32;
  let bp: *const u8 = v as *const _;
  let bs: &[u8] = unsafe {
      slice::from_raw_parts(
          bp,
          mem::size_of::<u32>()
      )
  };

  bs
}


impl Uri {
  fn as_bytes(&self)->Vec<u8> {

    let mut buf:Vec<u8> = vec![];
    buf.extend(to_bytes(&self._market));
    buf.extend(self._code.as_bytes());
    buf.extend(to_bytes(&self._line_no));

    buf
  }
}

extern crate db_key;
use db_key::*;

impl Key for Uri {
  fn from_u8(key: &[u8]) -> Self {
    Uri::from_u8(key)
  }

  fn as_slice<T, F: Fn(&[u8]) -> T>(&self, f: F) -> T {
    let mut s = self.as_bytes();
    //println!("bytes: {:?}", s);
    f(&mut s)
  }
}

fn bytes_to_u32(d:&[u8])->u32 {
  if d.len() == 4 {
    let value = (d[0] as u32) << 24 |
    (d[1] as u32) << 16 |
    (d[2] as u32) << 8 |
    (d[3] as u32);
    //println!("value:{:?}", value);
    return value;
  } else {
    0
  }
}

fn u32_to_bytes(v:u32)->[u8;4] {
  let mut bytes = [0;4];
  bytes[0] = (v>>24) as u8;
  bytes[1] = (v>>16) as u8;
  bytes[2] = (v>>8) as u8;
  bytes[3] = (v) as u8;
  
  bytes
}

fn get_line_no(db:&mut Database<Uri>, mkt:u32, code:&String)->u32 {
  let mut uri:Uri = Default::default();
  uri._code = code.clone();
  uri._market = mkt;
  uri._line_no = 0;

  let read_opts = ReadOptions::new();
  let res = db.get(read_opts, uri);

  let index = match res {
    Ok(data) => {
      if let Some(d/*Vec<u8>*/) = data {
        println!("Ok, got it: {:?}", d);
        bytes_to_u32(&d)
      } else {
        0
      }
    }
    Err(e) => { 0 }
  };

  index
}

use std::io;
fn set_line_no(db:&mut Database<Uri>, mkt:u32, code:&String, line:u32)->io::Result<()> {
  let mut uri:Uri = Default::default();
  uri._code = code.clone();
  uri._market = mkt;
  uri._line_no = 0;

  let write_opts = WriteOptions::new();
  if let Err(e) = db.put(write_opts, uri.clone(), &u32_to_bytes(line)) {
    panic!("failed to write db: {:?}", e)
  }
  println!("Success to write lineno: {:?}, {}", uri, line);
  Ok(())
}

fn main2() {
  let mut options = Options::new();
  options.create_if_missing = true;
  let mut database:Database<Uri> = match Database::open(Path::new("db"), options) {
      Ok(db) => { db },
      Err(e) => { panic!("failed to open database: {:?}", e) }
  };

  let mkt = 2u32;
  let stock_code = "000333".to_owned();
  set_line_no(&mut database, mkt, &stock_code, 1).unwrap();
  let mut line_no = get_line_no(&mut database, mkt, &stock_code);
  println!("get line_no: {}", line_no);
}

fn main() {

  let mut options = Options::new();
  options.create_if_missing = true;
  let mut database:Database<Uri> = match Database::open(Path::new("db"), options) {
      Ok(db) => { db },
      Err(e) => { panic!("failed to open database: {:?}", e) }
  };

  for _ in 0..5 {
    let mkt = 2u32;
    let stock_code = "000333".to_owned();
    let mut line_no = get_line_no(&mut database, mkt, &stock_code);
    println!("get line_no: {}", line_no);

    line_no += 1;
    
    let mut line:KLine = Default::default();
    line._close_price = line_no;
    let mut uri:Uri = Default::default();
    uri._code = stock_code.clone();
    uri._market = mkt;
    uri._line_no = line_no;

    let write_opts = WriteOptions::new();
    if let Err(e) = database.put(write_opts, uri.clone(), line.as_bytes()) {
      panic!("failed to write to database: {:?}", e)
    };
    println!("write kline:{:?}", uri);

    set_line_no(&mut database, mkt, &stock_code, line_no).unwrap();

    for i in 1..line_no+1 {
      println!("read {}", i);
      let mut uri2 = uri.clone();
      uri2._line_no = i;
      let read_opts = ReadOptions::new();
      let res = database.get(read_opts, uri2);

      match res {
        Ok(data) => {
          if let Some(d) = data {
            println!("Get lineno:{}, value:{:?}", i, d)
          }
        }
        Err(e) => { panic!("failed reading data: {:?}", e) }
      }
    }
  }
  
}