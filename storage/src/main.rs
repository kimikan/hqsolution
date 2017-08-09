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
    Uri::default()
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

fn add_line_no(db:&mut Database, mkt:u32, code:&String)->u32 {
  let mut uri:Uri = Default::default();
  uri._code = code.clone();
  uri._market = mkt;
  uri._line_no = 0;

  let read_opts = ReadOptions::new();
  let res = database.get(read_opts, uri.as_bytes());

  let index = match res {
    Ok(data) => {
      
    }
    Err(e) => { panic!("failed reading data: {:?}", e) }
  }
}

fn main() {

  let mut options = Options::new();
  options.create_if_missing = true;
  let mut database = match Database::open(Path::new("db"), options) {
      Ok(db) => { db },
      Err(e) => { panic!("failed to open database: {:?}", e) }
  };


  let line:KLine = Default::default();

  let write_opts = WriteOptions::new();
  match database.put(write_opts, uri.as_bytes(), to_bytes(&1)) {
      Ok(_) => { () },
      Err(e) => { panic!("failed to write to database: {:?}", e) }
  };

  let read_opts = ReadOptions::new();
  let res = database.get(read_opts, 1);

  match res {
    Ok(data) => {
      assert!(data.is_some());
      assert_eq!(data, Some(vec![1]));
    }
    Err(e) => { panic!("failed reading data: {:?}", e) }
  }

  let read_opts = ReadOptions::new();
  let mut iter = database.iter(read_opts);
  let entry = iter.next();
  assert_eq!(
    entry,
    Some((1, vec![1]))
  );
}