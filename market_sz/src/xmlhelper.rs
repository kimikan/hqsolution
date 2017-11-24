use xml;

use std::fs::File;
use std::io::BufReader;

use xml::reader::{EventReader, XmlEvent};
use std::io;
use std::collections::HashMap;

fn indent(size: usize) -> String {
    const INDENT: &'static str = "    ";
    (0..size).map(|_| INDENT)
             .fold(String::with_capacity(size*INDENT.len()), |r, s| r + s)
}

use t2sdk::StockRecord;
pub fn parse_static_file(file_name : &str, stocks : &mut HashMap<String, StockRecord>) ->io::Result<()> {
    let file = File::open(file_name)?;
    let file = BufReader::new(file);

    let parser = EventReader::new(file);
    let mut depth = 0;
    let mut code = String::default();
    let mut symbol = String::default();
    let mut pre_close_px = 0f32;
    let mut stock_status = 0u32;

    let mut parent_name = String::default();

    for e in parser {
        match e {
            Ok(XmlEvent::StartElement { name, .. }) => {
                depth += 1;
                if depth == 3 {
                    parent_name = name.local_name;
                } else if depth == 4 {
                    parent_name = name.local_name;
                }
            }
            Ok(XmlEvent::EndElement { name }) => {
                depth -= 1;
                parent_name == String::default();
                if depth == 1 {
                    if name.local_name.eq("Security") {
                        if stock_status > 0 {
                            //print all of the stop stocks for debuging usage
                            println!("{:?}, {}, {}, {}", code, symbol, pre_close_px, stock_status);
                        }

                         let value = stocks.entry(code.clone()).or_insert(StockRecord::default());
                        value._stock_code = code.clone();
                        value._stock_name = symbol.clone();
                        value._pre_close_px = (pre_close_px * 1000f32) as u32;

                        if stock_status == 1 {
                            value._trade_status = 9;
                        }
                        code = String::default();
                        symbol = String::default();
                        pre_close_px = 0f32;
                        stock_status = 0u32;
                    }
                }
            }
            Ok(XmlEvent::Characters(text))=>{
                if depth == 3 {
                    match parent_name.as_str() {
                        "SecurityID"=>{
                            code = text.trim().to_owned();
                        }
                        "Symbol"=>{
                            symbol = text.trim().to_owned();
                        }
                        "PrevClosePx"=>{
                            pre_close_px = text.parse().unwrap_or(0f32);
                        }
                        "SecurityStatus"=>{
                            stock_status = text.parse().unwrap_or(0);
                        }
                        _=>{}
                    };
                } //end if depth = 3
                else if depth == 4 {
                    match parent_name.as_str() {
                        "Status"=>{
                            stock_status = text.parse().unwrap_or(0);
                        }
                        _=>{}
                    };
                }
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
            _ => {}
        }
    } //end fn?

    Ok(())
}