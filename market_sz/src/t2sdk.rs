

use std::io;


#[derive(Debug, Clone)]
pub struct StockRecord {
    pub _stock_code : String,
    //2:index, 3:Fund, 4:Debt, 1:Stock
    pub _stock_type : u32,
    pub _stock_name : String,

	pub _last_px : u32,

	pub _trade_amount : i64,
    pub _trade_balance : i64,
    pub _trade_status : u32,
	pub _open_px : u32,
	pub _close_px : u32,
	pub _pre_close_px : u32,
	pub _high_px : u32,
	pub _low_px : u32,

	pub _date : u32,
    pub _time : u32,	
	
    pub _sale_pxs : [u32; 5],
    pub  _sale_amounts:[u32; 5],
    pub _buy_pxs : [u32; 5],
    pub _buy_amounts : [u32; 5],

	pub _market_value : u64,
    
	pub _pe_rate : u32,
    pub _dynamic_pe : u32,
    
	pub _first_date : u32,
    pub _first_px : u32,
    
    pub _line_no : u32,

    pub _total_shares : i64,
    pub _nonstrict_shares : i64,
}


impl StockRecord {

    fn push(&self)->io::Result<()> {
        println!("{:?}", self);
        Ok(())
    }
}

impl Default for StockRecord {

    fn default()->StockRecord {

        StockRecord {
         _stock_code : Default::default(),
        //2:index, 3:Fund, 4:Debt, 1:Stock
        _stock_type : 0u32,
        _stock_name : Default::default(),

        _last_px : 0u32,

        _trade_amount : 0,
        _trade_balance : 0,
        _trade_status : 0u32,
        _open_px : 0u32,
        _close_px : 0u32,
        _pre_close_px : 0u32,
        _high_px : 0u32,
        _low_px : 0u32,

        _date : 0u32,
        _time : 0u32,	
        
        _sale_pxs : [0; 5],
        _sale_amounts:[0; 5],
        _buy_pxs : [0; 5],
        _buy_amounts : [0; 5],

        _market_value : 0u64,
        
        _pe_rate : 0u32,
        _dynamic_pe : 0u32,
        
        _first_date : 0u32,
        _first_px : 0u32,
        _line_no : 0u32,
        _total_shares : 0i64,
        _nonstrict_shares : 0i64,
        }//end stock record
    }
}

pub fn push_stock_status(code : &str, trade_status : u32, time : u32)->io::Result<()> {

    Ok(())
}

pub fn push_market_datetime(dt : i64)->io::Result<()> {
    let date  = ( dt / 1000000000) as u32;
    let time = (( dt /1000) % 1000000) as u32;

    push_market_time(date, time)
}

pub fn push_market_time(date : u32, time : u32)->io::Result<()> {

    Ok(())
}

pub fn push_market_status(date : u32, time : u32, trade_status:u32)->io::Result<()> {

    Ok(())
}

pub fn push_stock(stock : &StockRecord)->io::Result<()> {
    println!("{:?}", stock);
    stock.push()
}

pub fn push_debt(stock : &StockRecord)->io::Result<()> {

    stock.push()
}


pub fn push_fund(stock : &StockRecord)->io::Result<()> {

    stock.push()
}


pub fn push_index(stock : &StockRecord)->io::Result<()> {

    stock.push()
}