
use chrono;
use tiberius;
use futures::Future;
use tiberius::stmt::ResultStreamExt;
use tiberius::{SqlConnection, TdsResult};

use std::collections::HashMap;
use std;
use t2sdk::StockRecord;

use std::io;
use tokio_core::reactor::Core;
//use
pub const CONNSTRING: &str = "server=tcp:139.196.143.124,1433;
    Database=JYDB;Uid=sa;Pwd=Hznb@123;TrustServerCertificate=true;";

pub const QUERYSTRING: &str = "SELECT b.SecuCode,a.IssuePrice,a.listdate
    FROM LC_AShareIPO a,SecuMain b
    where a.InnerCode = b.InnerCode and a.listdate>0 and b.SecuMarket=90
    order by b.SecuCode desc;";

pub const HQSTRING: &str = "select * from NiubangHqTable
    where SecuMarket=90 and NonRestrictedShares is not null;";

pub struct Sqlserver {
    _core: Core,
}

impl Sqlserver {
    pub fn new() -> Option<Sqlserver> {
        let core = Core::new();
        if let Ok(c) = core {
            return Some(Sqlserver { _core: c });
        }

        None
    }

    pub fn update(&mut self, stocks: &mut HashMap<String, StockRecord>) -> io::Result<()> {

        let conn_new = SqlConnection::connect(self._core.handle(), CONNSTRING);

        let future = conn_new.and_then(|conn| {
            conn.simple_query(QUERYSTRING)
                .for_each_row(|row| {
                    let code: Option<&str> = row.try_get(0)?;
                    let issue_px: Option<f64> = row.try_get(1).unwrap_or(None);

                    let first_date: Option<chrono::NaiveDateTime> = row.try_get(2).unwrap_or(None);
                    //println!("{:?}, {:?}, {:?}", code, issue_px, first_date);

                    if let Some(c) = code {
                        let px = issue_px.unwrap_or_default();

                        if let Some(d) = first_date {

                            let value = stocks
                                .entry(c.to_owned())
                                .or_insert(StockRecord::default());
                            value._stock_code = c.to_owned();
                            value._first_px = (px * 1000f64) as u32;

                            use chrono::Datelike;
                            let year = d.year();
                            value._first_date = (year as u32) * 10000 + d.month() * 100 + d.day();

                            return Ok(());
                        }
                    }
                    //Err(tiberius::TdsError::Canceled)

                    Ok(())
                })
        });

        if let std::result::Result::Err(e) = self._core.run(future) {
            println!("{:?}", e);
        }

        Ok(())
    }

    pub fn update2(&mut self, stocks: &mut HashMap<String, StockRecord>) -> io::Result<()> {

        let conn_new = SqlConnection::connect(self._core.handle(), CONNSTRING);

        let future = conn_new.and_then(|conn| {
            conn.simple_query(HQSTRING)
                .for_each_row(|row| {
                    let code: Option<&str> = row.try_get(1).unwrap_or(None);
                    let name: Option<&str> = row.try_get(2).unwrap_or(None);
                    let total_shares: Option<&str> = row.try_get(4).unwrap_or(None);
                    let nonstrict_shares: Option<&str> = row.try_get(5).unwrap_or(None);
                    let eps: Option<f64> = row.try_get(6).unwrap_or(None);
                    let dynamic_eps: Option<f64> = row.try_get(8).unwrap_or(None);

                    if let Some(c) = code {
                        let n = name.unwrap_or_default();
                        let ts = total_shares.unwrap_or_default();
                        //let ns = nonstrict_shares.unwrap_or_default();
                        //let es = eps.unwrap_or(0f64);
                        //let des = dynamic_eps.unwrap_or(0f64);

                        let value = stocks
                            .entry(c.to_owned())
                            .or_insert(StockRecord::default());
                        value._stock_code = c.to_owned();
                        value._stock_name = n.to_owned();
                        value._total_shares = ts.parse().unwrap_or_default();

                        if let Some(ns) = nonstrict_shares {
                            value._nonstrict_shares = ns.parse().unwrap_or_default();
                        }

                        if let Some(es) = eps {
                            value._pe_rate = (es * 1000f64) as u32;
                        }

                        if let Some(des) = dynamic_eps {
                            value._dynamic_pe = (des * 1000f64) as u32;
                        }
                        //println!("{:?}", value);
                        return Ok(());
                    }

                    //Err(tiberius::TdsError::Canceled)
                    Ok(())
                })
        });

        if let std::result::Result::Err(e) = self._core.run(future) {
            println!("-----------x{:?}", e);
        }

        Ok(())
    }
}
