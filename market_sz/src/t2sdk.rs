

use std::io;
use std::ffi::CString;
use std::os::raw::c_char;

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

use utils;

impl StockRecord {

    fn push(&self, ctx : &mut T2Context, func_no : i32)->io::Result<()> {
        let mut msg = T2Message::new();
        msg.set_packet_type(interoper::REQUEST);
        msg.set_function_no(func_no);
        info!("{:?}", self);

        unsafe {
            t2message_beginpack(msg._message);
            t2message_addfield(msg._message,  CString::new("stock_code").unwrap().as_ptr());
            t2message_addfield(msg._message,  CString::new("stock_type").unwrap().as_ptr());
            t2message_addfield(msg._message,  CString::new("stock_name").unwrap().as_ptr());
            t2message_addfield(msg._message,  CString::new("exchange_type").unwrap().as_ptr());
            t2message_addfield(msg._message,  CString::new("last_price").unwrap().as_ptr());
            t2message_addfield(msg._message,  CString::new("business_amount").unwrap().as_ptr());
            t2message_addfield(msg._message,  CString::new("split_amount").unwrap().as_ptr());
            t2message_addfield(msg._message, CString::new("business_balance").unwrap().as_ptr());
            t2message_addfield(msg._message, CString::new("open_price").unwrap().as_ptr());
            t2message_addfield(msg._message, CString::new("close_price").unwrap().as_ptr());
            t2message_addfield(msg._message,  CString::new("pre_close_price").unwrap().as_ptr());
            t2message_addfield(msg._message,  CString::new("high_price").unwrap().as_ptr());
            t2message_addfield(msg._message,  CString::new("low_price").unwrap().as_ptr());
            t2message_addfield(msg._message,  CString::new("record_time").unwrap().as_ptr());
            t2message_addfield(msg._message,  CString::new("line_no").unwrap().as_ptr());
            t2message_addfield(msg._message,  CString::new("market_date").unwrap().as_ptr());

            t2message_addfield(msg._message, CString::new("sale_price5").unwrap().as_ptr());
            t2message_addfield(msg._message, CString::new("sale_amount5").unwrap().as_ptr());
            t2message_addfield(msg._message, CString::new("sale_price4").unwrap().as_ptr());
            t2message_addfield(msg._message, CString::new("sale_amount4").unwrap().as_ptr());
            t2message_addfield(msg._message, CString::new("sale_price3").unwrap().as_ptr());
            t2message_addfield(msg._message, CString::new("sale_amount3").unwrap().as_ptr());
            t2message_addfield(msg._message, CString::new("sale_price2").unwrap().as_ptr());
            t2message_addfield(msg._message, CString::new("sale_amount2").unwrap().as_ptr());
            t2message_addfield(msg._message, CString::new("sale_price1").unwrap().as_ptr());
            t2message_addfield(msg._message, CString::new("sale_amount1").unwrap().as_ptr());
            t2message_addfield(msg._message, CString::new("buy_price1").unwrap().as_ptr());
            t2message_addfield(msg._message,  CString::new("buy_amount1").unwrap().as_ptr());
            t2message_addfield(msg._message, CString::new("buy_price2").unwrap().as_ptr());
            t2message_addfield(msg._message,  CString::new("buy_amount2").unwrap().as_ptr());
            t2message_addfield(msg._message, CString::new("buy_price3").unwrap().as_ptr());
            t2message_addfield(msg._message,  CString::new("buy_amount3").unwrap().as_ptr());
            t2message_addfield(msg._message, CString::new("buy_price4").unwrap().as_ptr());
            t2message_addfield(msg._message,  CString::new("buy_amount4").unwrap().as_ptr());
            t2message_addfield(msg._message, CString::new("buy_price5").unwrap().as_ptr());
            t2message_addfield(msg._message,  CString::new("buy_amount5").unwrap().as_ptr());

            t2message_addfield(msg._message,  CString::new("market_value").unwrap().as_ptr());
            t2message_addfield(msg._message,  CString::new("turnover_ratio").unwrap().as_ptr());
            t2message_addfield(msg._message,  CString::new("pe_rate").unwrap().as_ptr());
            t2message_addfield(msg._message,  CString::new("dynamic_pe_rate").unwrap().as_ptr());
            t2message_addfield(msg._message,  CString::new("first_date").unwrap().as_ptr());
            t2message_addfield(msg._message, CString::new("issue_price").unwrap().as_ptr());
            t2message_addfield(msg._message, CString::new("trade_status").unwrap().as_ptr());
            t2message_addfield(msg._message, CString::new("margin_status").unwrap().as_ptr());

            t2message_addstr(msg._message, CString::new(self._stock_code.as_str()).unwrap().as_ptr());
            t2message_addint(msg._message, self._stock_type as i64);

            let name = utils::string_to_gb2312(&self._stock_name);
            
            t2message_addstr(msg._message, CString::new(name).unwrap().as_ptr());
            t2message_addchar(msg._message, b'2');
            t2message_addint(msg._message, self._last_px as i64);
            
            t2message_addstr(msg._message, CString::new(self._trade_amount.to_string().as_str()).unwrap().as_ptr());
            t2message_addstr(msg._message, CString::new(self._trade_amount.to_string().as_str()).unwrap().as_ptr());
            t2message_addstr(msg._message, CString::new(self._trade_balance.to_string().as_str()).unwrap().as_ptr());
            t2message_addint(msg._message, self._open_px as i64);
            t2message_addint(msg._message, self._close_px as i64);
            t2message_addint(msg._message, self._pre_close_px as i64);
            t2message_addint(msg._message, self._high_px as i64);
            t2message_addint(msg._message, self._low_px as i64);
            t2message_addint(msg._message, self._time as i64);
            t2message_addint(msg._message, utils::get_line_number(self._time) as i64);
            t2message_addint(msg._message, self._date as i64);

            t2message_addint(msg._message, self._sale_pxs[4] as i64);
            t2message_addint(msg._message, self._sale_amounts[4] as i64);
            t2message_addint(msg._message, self._sale_pxs[3] as i64);
            t2message_addint(msg._message, self._sale_amounts[3] as i64);
            t2message_addint(msg._message, self._sale_pxs[2] as i64);
            t2message_addint(msg._message, self._sale_amounts[2] as i64);
            t2message_addint(msg._message, self._sale_pxs[1] as i64);
            t2message_addint(msg._message, self._sale_amounts[1] as i64);
            t2message_addint(msg._message, self._sale_pxs[0] as i64);
            t2message_addint(msg._message, self._sale_amounts[0] as i64);

            t2message_addint(msg._message, self._buy_pxs[0] as i64);
            t2message_addint(msg._message, self._buy_amounts[0] as i64);
            t2message_addint(msg._message, self._buy_pxs[1] as i64);
            t2message_addint(msg._message, self._buy_amounts[1] as i64);
            t2message_addint(msg._message, self._buy_pxs[2] as i64);
            t2message_addint(msg._message, self._buy_amounts[2] as i64);
            t2message_addint(msg._message, self._buy_pxs[3] as i64);
            t2message_addint(msg._message, self._buy_amounts[3] as i64);
            t2message_addint(msg._message, self._buy_pxs[4] as i64);
            t2message_addint(msg._message, self._buy_amounts[4] as i64);

            t2message_addint(msg._message, self._market_value as i64);
            t2message_addint(msg._message, 0i64);//todo: change rate
            t2message_addint(msg._message, self._pe_rate as i64);
            t2message_addint(msg._message, self._dynamic_pe as i64);
            t2message_addint(msg._message, self._first_date as i64);
            t2message_addint(msg._message, self._first_px as i64);
            t2message_addint(msg._message, self._trade_status as i64);
            t2message_addint(msg._message, 0i64);//todo: margin status

            t2message_endpack(msg._message);

            let ret = send_message(ctx._context, msg._message);

            if ret <= 0 {
                println!("Push stock failed: {:}", ret);
            }
        }
        
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

use interoper;
use interoper::*;

use log::*;
pub fn push_stock_status(ctx : &mut T2Context, code : &str, trade_status : u32, time : u32)->io::Result<()> {
    //not used.
    Ok(())
}

pub fn push_market_datetime(ctx : &mut T2Context, dt : i64)->io::Result<()> {
    let date  = ( dt / 1000000000) as u32;
    let time = (( dt /1000) % 1000000) as u32;

    push_market_time(ctx, date, time)
}

fn get_market_status(time : u32)->u32 {
    
    let status : u32 = match time {
        85800...91459=>1,//before open
        91500...92500=>2,//auction
        92500...92959=>3,
        93000...112959=>4,//trading
        113000...125959=>5,//noon closing
        130000...145700=>4,
        145700...150000=>8,
        _=>6,//Stop
    };

    status
}

static mut __DATE : u32 = 0;
static mut __TIME : u32 = 0;

pub fn push_market_time(ctx : &mut T2Context, date : u32, time : u32)->io::Result<()> {
    unsafe {
        if __DATE == date && __TIME == time {
            return Ok(());
        }

        __DATE = date;
        __TIME = time;
    }

    let mut msg = T2Message::new();
    msg.set_packet_type(interoper::REQUEST);
    msg.set_function_no(2206);

    unsafe {
        t2message_beginpack(msg._message);
        t2message_addfield(msg._message,  CString::new("exchange_type").unwrap().as_ptr());
        t2message_addfield(msg._message,  CString::new("market_date").unwrap().as_ptr());
        t2message_addfield(msg._message,  CString::new("market_time").unwrap().as_ptr());
        t2message_addfield(msg._message, CString::new("line_no").unwrap().as_ptr());
        t2message_addfield(msg._message, CString::new("market_status").unwrap().as_ptr());

        t2message_addchar(msg._message, b'2');
        t2message_addint(msg._message, date as i64);
        t2message_addint(msg._message, time as i64);
        
        use utils;
        t2message_addint(msg._message, utils::get_line_number(time) as i64);
        t2message_addint(msg._message, get_market_status(time) as i64);
        t2message_endpack(msg._message);
        
        let ret = send_message(ctx._context, msg._message);

        if ret <= 0 {
            println!("Push time failed: {:}", ret);
        }
    }

    let status = get_market_status(time);
    push_market_status(ctx, date, time, status)?;
    Ok(())
}

static mut __MARKET_STATUS : u32 = 0;
pub fn push_market_status(ctx : &mut T2Context, date : u32, time : u32, trade_status : u32)->io::Result<()> {
    unsafe {
        if __MARKET_STATUS == trade_status {
            return Ok(());
        }
        __MARKET_STATUS = trade_status;
    }

    
    let mut msg = T2Message::new();
    msg.set_packet_type(interoper::REQUEST);
    msg.set_function_no(2206);

    unsafe {
        t2message_beginpack(msg._message);
        t2message_addfield(msg._message,  CString::new("exchange_type").unwrap().as_ptr());
        t2message_addfield(msg._message,  CString::new("market_date").unwrap().as_ptr());
        t2message_addfield(msg._message,  CString::new("market_time").unwrap().as_ptr());
        t2message_addfield(msg._message, CString::new("line_no").unwrap().as_ptr());
        t2message_addfield(msg._message, CString::new("market_status").unwrap().as_ptr());
        t2message_addfield(msg._message, CString::new("market_time_old").unwrap().as_ptr());

        t2message_addchar(msg._message, b'2');
        t2message_addint(msg._message, date as i64);
        t2message_addint(msg._message, time as i64);
        
        use utils;
        t2message_addint(msg._message, utils::get_line_number(time) as i64);
        t2message_addint(msg._message, get_market_status(time) as i64);
        t2message_addint(msg._message, time as i64);

        t2message_endpack(msg._message);

        let ret = send_message(ctx._context, msg._message);

        if ret <= 0 {
            println!("Push time failed: {:}", ret);
        }
    }
    
    Ok(())
}

pub fn push_stock(ctx : &mut T2Context, stock : &StockRecord)->io::Result<()> {
    //println!("{:?}", stock);
    stock.push(ctx, 2202)
}

pub fn push_debt(ctx : &mut T2Context, stock : &StockRecord)->io::Result<()> {

    stock.push(ctx, 2202)
}


pub fn push_fund(ctx : &mut T2Context, stock : &StockRecord)->io::Result<()> {

    stock.push(ctx, 2205)
}

pub fn push_index(ctx : &mut T2Context, stock : &StockRecord)->io::Result<()> {

    stock.push(ctx, 2203)
}