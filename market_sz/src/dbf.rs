
use std::collections::HashMap;

struct Record {
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
}

struct Dbf {
    _records  : HashMap<String, Record>,
}
