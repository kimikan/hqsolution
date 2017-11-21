
use std::collections::HashMap;

use utils;
use std::mem;

pub struct Record {
    //offset from the begin of the file
    _offset : u32,

    _stock_code : String,
    //2:index, 3:Fund, 4:Debt, 1:Stock
    _stock_type : u32,
    _stock_name : String,

	_last_px : u32,

	_trade_amount : u64,
    _trade_balance : u64,

	_open_px : u32,
	_close_px : u32,
	_pre_close_px : u32,
	_high_px : u32,
	_low_px : u32,

	_date : u32,
    _time : u32,	
	
    _sale_pxs : [u32; 5],
    _sale_amounts:[u32; 5],
    _buy_pxs : [u32; 5],
    _buy_amounts : [u32; 5],

	_market_value : u64,
    
	_pe_rate : u32,
    _dynamic_pe : u32,
    
	_first_date : u32,
    _first_px : u32,
}

impl Record {

    fn new(offset: u32)->Record {
        Record {
            _offset : offset,
            _stock_code : Default::default(),
            //2:index, 3:Fund, 4:Debt, 1:Stock
            _stock_type : 0u32,
            _stock_name : Default::default(),

            _last_px : 0u32,

            _trade_amount : 0u64,
            _trade_balance : 0u64,

            _open_px : 0u32,
            _close_px : 0u32,
            _pre_close_px : 0u32,
            _high_px : 0u32,
            _low_px : 0u32,

            _date : 0u32,
            _time : 0u32,	
            
            _sale_pxs : Default::default(),
            _sale_amounts : Default::default() ,
            _buy_pxs : Default::default(),
            _buy_amounts : Default::default(),

            _market_value : 0u64,
            
            _pe_rate : 0u32,
            _dynamic_pe : 0u32,
            
            _first_date : 0u32,
            _first_px : 0u32,
        }
    }

    fn sync_to_file(file : &String) {

    }
}


use std::fs::File;
use std::io::Read;

#[repr(C, packed)]
#[derive(Debug)]
struct DbfHead {
    _flag : u8,
    _year: u8,
    _month:u8,
    _day : u8,
    _record_count : i32, //little endian

    _offset : u16,//offset of the first record
    _record_len : u16,
    _reserved : [u8; 20],
}

impl DbfHead {

    fn new()->DbfHead {
        DbfHead {
            _flag : 3,
            _year : 0,
            _month : 0,
            _day : 0,
            _record_count : 0i32,
            _offset : 1153u16,

            _record_len : 352u16,
            _reserved : Default::default(),
        }
    }

    fn get_field_number(&self)->usize {
        (self._offset as usize - mem::size_of::<DbfHead>()) / mem::size_of::<DbfColumn>()
    }

    fn from(f : &mut File)->Option<DbfHead> {
        
        let mut buf :[u8;32] = [0; 32];
        let result = f.read_exact(&mut buf[..]);

        let head : DbfHead = unsafe {mem::transmute_copy(&buf)};
        match result {
            Ok(_)=>{
                Some(head)
            },
            Err(_)=> {
                None
            }
        }
    }
}

#[repr(C, packed)]
#[derive(Debug)]
struct DbfColumn {
    _col_name : [u8; 11],
    _col_type : u8,
    _offset : u32,//offset field in record
    _col_len : u8,
    _precision: u8,
    _reserved : [u8; 14],
}

pub struct Dbf {
    pub _records  : HashMap<String, Record>,
    pub _file : File,
    
}

impl Dbf {

    pub fn new(f : &str)->Option<Dbf> {
        
        let file = File::open(f);
        if let Ok(f) = file {
            return Some(Dbf {
                _records : Default::default(),
                _file : f,
            });
        }

        None
    }

    pub fn read_head(&mut self) {
        let f_o = DbfHead::from(&mut self._file);
        if let Some(f) = f_o {
            println!("{:?}, field_number: {:?}", f, f.get_field_number());
            
        }
    }
}