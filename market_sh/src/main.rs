
extern crate encoding;

mod utils;

use std::io;
use std::fs::{OpenOptions, File};
use std::io::{BufReader, BufRead, Read};
use std::collections::HashMap;
use std::time::SystemTime;

#[derive(Debug, Clone)]
struct BasicInfo {
    _code:String,
    _name:String,
    //100 per hand, or something
    _trade_volume:u64,
    //related deal money,
    _total_value_traded:f64,

    _pre_close_px:f32,
    _open_px:f32,
    _high_px:f32,
    _low_px:f32,
    _last_px:f32,
    _close_px:f32,

    _trade_phase_code:[u8;8],
     //HHMMSS
    _time:u32,
}

impl Default for BasicInfo {
    fn default()->BasicInfo {
        BasicInfo {
            _code:String::new(),
            _name:String::new(),
            _trade_volume:0u64,
            _total_value_traded:0f64,
            _pre_close_px:0f32,
            _open_px:0f32,
            _high_px:0f32,
            _low_px:0f32,
            _last_px:0f32,
            _close_px:0f32,

            _trade_phase_code:[0;8],
            _time:0,
        }
    }
}

impl BasicInfo {
    fn internal_from(line:&[u8])->Option<BasicInfo> {
        if line.len() < 128 {
            return None;
        }

        let mut basicinfo:BasicInfo = Default::default();
        
        let security_id = &line[6..12];
        let name = &line[13..21];
        if let Ok(id) = String::from_utf8(security_id.to_vec()) {
            basicinfo._code = id;
        } else {
            return None;
        }

        let refs = encoding::all::encodings();

        use encoding::DecoderTrap;
        let (name_result, _) = encoding::decode(name, DecoderTrap::Strict, refs[37]);
        if let Ok(n) = name_result {
            basicinfo._name = n;
            //println!("xxxx: {:?} {:?}",basicinfo._code, basicinfo._name);
        } else {
            return None;
        }
        
        let trade_vol = &line[22..38];
        
        if let Ok(value) = String::from_utf8(trade_vol.to_vec()) {
            //println!("*{:?}*", value);
            if let Ok(v) = value.trim().parse::<u64>() {
                basicinfo._trade_volume = v;
            } else {
                return None;
            }
        } else {
            return None;
        }

        //total value traded
        let total_traded = &line[39..55];
        if let Ok(value) = String::from_utf8(total_traded.to_vec()) {
            if let Ok(v) = value.trim().parse::<f64>() {
                basicinfo._total_value_traded = v;
            } else {
                return None;
            }
        } else {
            return None;
        }
        
        //prev close px
        let prev_px = &line[56..67];
        if let Ok(value) = String::from_utf8(prev_px.to_vec()) {
            if let Ok(v) = value.trim().parse::<f32>() {
                basicinfo._pre_close_px = v;
            } else {
                return None;
            }
        } else {
            return None;
        }

        //open px
        let open_px = &line[68..79];
        if let Ok(value) = String::from_utf8(open_px.to_vec()) {
            if let Ok(v) = value.trim().parse::<f32>() {
                basicinfo._open_px = v;
            } else {
                return None;
            }
        } else {
            return None;
        }

         //high px
        let high_px = &line[80..91];
        if let Ok(value) = String::from_utf8(high_px.to_vec()) {
            if let Ok(v) = value.trim().parse::<f32>() {
                basicinfo._high_px = v;
            } else {
                return None;
            }
        } else {
            return None;
        }
        //low px
        let low_px = &line[92..103];
        if let Ok(value) = String::from_utf8(low_px.to_vec()) {
            if let Ok(v) = value.trim().parse::<f32>() {
                basicinfo._low_px = v;
            } else {
                return None;
            }
        } else {
            return None;
        }

        //last px
        let last_px = &line[104..115];
        println!("last_px: {:?}", last_px);
        if let Ok(value) = String::from_utf8(last_px.to_vec()) {
            if let Ok(v) = value.trim().parse::<f32>() {
                basicinfo._last_px = v;
            } else {
                return None;
            }
        } else {
            return None;
        }

        //close px
        let close_px = &line[116..127];
        if let Ok(value) = String::from_utf8(close_px.to_vec()) {
            if let Ok(v) = value.trim().parse::<f32>() {
                basicinfo._close_px = v;
            } else {
                return None;
            }
        } else {
            return None;
        }

        Some(basicinfo)
    }

    //for fund:offset is 378+24, sum is 424
    //for others offset is 378, sum is 400
    fn from2(line:&[u8], offset:usize, sum:usize)->Option<BasicInfo> {
        if line.len() < sum {
            return None;
        }
        let mut basicinfo = BasicInfo::internal_from(line);

        if let Some(mut info) = basicinfo {
            //phase code
            info._trade_phase_code.copy_from_slice(&line[offset..offset + 8]);
            //time stamp
            let hour = &line[offset + 9..offset + 11];
            if let Ok(value) = String::from_utf8(hour.to_vec()) {
                if let Ok(v) = value.trim().parse::<u32>() {
                    info._time = v * 10000;
                } else {
                    return None;
                }
            } else {
                return None;
            }
            let mins = &line[offset + 12..offset + 14];
            if let Ok(value) = String::from_utf8(mins.to_vec()) {
                if let Ok(v) = value.parse::<u32>() {
                    info._time += v * 100;
                } else {
                    return None;
                }
            } else {
                return None;
            }
            
            let secs = &line[offset + 15..offset + 17];
            if let Ok(value) = String::from_utf8(secs.to_vec()) {
                if let Ok(v) = value.parse::<u32>() {
                    info._time += v;
                } else {
                    return None;
                }
            } else {
                return None;
            } //end let?

            return Some(info);
        }
        
        None
    }

    //other format
    fn from(line:&[u8])->Option<BasicInfo> {
        if line.len() < 146 {
            return None;
        }
        let mut basicinfo = BasicInfo::internal_from(line);
        if let Some(mut info) = basicinfo {
            //phase code
            info._trade_phase_code.copy_from_slice(&line[128..136]);
            //time stamp
            let hour = &line[137..139];
            if let Ok(value) = String::from_utf8(hour.to_vec()) {
                if let Ok(v) = value.trim().parse::<u32>() {
                    info._time = v * 10000;
                } else {
                    return None;
                }
            } else {
                return None;
            }
        
            let mins = &line[140..142];
            if let Ok(value) = String::from_utf8(mins.to_vec()) {
                if let Ok(v) = value.parse::<u32>() {
                    info._time += v * 100;
                } else {
                    return None;
                }
            } else {
                return None;
            }

            let secs = &line[143..145];
            if let Ok(value) = String::from_utf8(secs.to_vec()) {
                if let Ok(v) = value.parse::<u32>() {
                    info._time += v;
                } else {
                    return None;
                }
            } else {
                return None;
            } //end let?

            return Some(info);
        }
        
        None
    }
}

#[derive(Debug, Clone)]
struct Index {
    _info:BasicInfo,
}

impl Index {
    fn from(buf:&[u8])->Option<Index> {
        let info = BasicInfo::from(buf);

        if let Some(information) = info {
            return Some(
                Index {
                    _info:information,
                }
            );
        }
        
        None
    }//end new()
}

#[derive(Debug, Clone)]
struct Stock {
    _info:BasicInfo,

    _buy_pxs:[f32;5],
    _buy_volumes:[u32;5],

    _sell_pxs:[f32;5],
    _sell_volumes:[u32;5],
}

impl Default for Stock {
    fn default()->Stock {
        Stock {
            _info:Default::default(),
            _buy_pxs:[0f32;5],
            _buy_volumes:[0;5],
            _sell_pxs:[0f32;5],
            _sell_volumes:[0;5],
        }
    }//default impl
}

impl Stock {

    fn from(buf:&[u8])->Option<Stock> {
        Stock::from2(buf, 378, 400)
    }

    fn from2(buf:&[u8], offset:usize, sum:usize)->Option<Stock> {
        let mut stock:Stock = Default::default();
        let info_op = BasicInfo::from2(buf, offset, sum);

        if let Some(info) = info_op {
            stock._info = info;
        }

        let mut start_offset = 128;
        for i in 0..5 {
             let buy_px1 = &buf[start_offset..start_offset + 11];
            if let Ok(value) = String::from_utf8(buy_px1.to_vec()) {
                if let Ok(v) = value.trim().parse::<f32>() {
                    stock._buy_pxs[i] = v;
                } else {
                    return None;
                }
            } else {
                return None;
            }

            let buy_vol1 = &buf[start_offset + 12..start_offset + 24];
            if let Ok(value) = String::from_utf8(buy_vol1.to_vec()) {
                if let Ok(v) = value.trim().parse::<u32>() {
                    stock._buy_volumes[i] = v;
                } else {
                    return None;
                }
            } else {
                return None;
            }

            let sell_px1 = &buf[start_offset + 25..start_offset + 36];
            if let Ok(value) = String::from_utf8(sell_px1.to_vec()) {
                if let Ok(v) = value.trim().parse::<f32>() {
                    stock._sell_pxs[i] = v;
                } else {
                    return None;
                }
            } else {
                return None;
            }

            let sell_vol1 = &buf[start_offset + 37..start_offset + 49];
            if let Ok(value) = String::from_utf8(sell_vol1.to_vec()) {
                if let Ok(v) = value.trim().parse::<u32>() {
                    stock._sell_volumes[i] = v;
                } else {
                    return None;
                }
            } else {
                return None;
            }
            start_offset += 50;
        }

        Some(stock)
    }
}

use std::ops::{Deref, DerefMut};
#[derive(Debug, Clone)]
struct Debt {
    _item:Stock,
}

impl Debt {
    fn from(buf:&[u8])->Option<Debt> {
        let stock = Stock::from(buf);
        
        if let Some(s) = stock {
            return Some(Debt{
                _item:s,
            })
        }
        
        None
    }//end from?
}

impl Deref for Debt {
    type Target = Stock;

    fn deref<'a>(&'a self) -> &'a Stock {
        &self._item
    }
}

impl DerefMut for Debt {
    fn deref_mut<'a>(&'a mut self) -> &'a mut Stock {
        &mut self._item
    }
}

#[derive(Debug, Clone)]
struct Fund {
    _item:Stock,
}

impl Fund {
    fn from(buf:&[u8])->Option<Fund> {
        let stock = Stock::from2(buf, 402, 424);
        println!("------ {} {:?}", buf.len(), stock);

        if let Some(s) = stock {
            return Some(Fund{
                _item:s,
            })
        }
        
        None
    }//end from?
}

impl Deref for Fund {
    type Target = Stock;

    fn deref<'a>(&'a self) -> &'a Stock {
        &self._item
    }
}

impl DerefMut for Fund {
    fn deref_mut<'a>(&'a mut self) -> &'a mut Stock {
        &mut self._item
    }
}

#[derive(Debug, Clone)]
enum DataItem {
    IndexType(Index),
    FundType(Fund),
    DebtType(Debt),
    StockType(Stock),

    None,
}

struct Context {
    _prev_len:usize,
    _stocks:HashMap<String, DataItem>,
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
    let id = String::from_utf8_lossy(stream_id);
    
    let mut item = DataItem::None;

    if id == "MD001" {
        let index = Index::from(line);
        if let Some(i) = index {
            item = DataItem::IndexType(i);
        }
    } else if id == "MD002" {
         let stock = Stock::from(line);
        if let Some(i) = stock {
             /*if i._info._last_px > 0f32 {
                 panic!("{:?}", i);
            } */
            item = DataItem::StockType(i);
        }
    } else if id == "MD003" {
        let debt = Debt::from(line);
        if let Some(i) = debt {
            item = DataItem::DebtType(i);
        }
    } else if id == "MD004" {
        let fund = Fund::from(line);
        if let Some(i) = fund {
            item = DataItem::FundType(i);
        }
       
    }
    println!("{:?}, {:?}", id, item);
    //println!("{:?}, {:?}", String::from_utf8(stream_id).unwrap()
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
    let ctx = Context::new();
    if let Err(result) = process_file(ctx, "MKTDT00.TXT") {
        println!("{}", result);
    }
}
