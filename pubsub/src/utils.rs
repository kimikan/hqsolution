

use std::slice;
use std::mem;

struct Stock {

    _code:[u8; 8],
    _name:[u8; 20],

    _last_price:u32,
}

//Stock operation
impl Stock {

    fn marshal(&mut self)->&mut [u8] {
        unsafe {
            slice::from_raw_parts_mut((self as *mut Stock) as *mut u8, mem::size_of::<Stock>())
        }

    }

    fn from(buf:&[u8])->Option<Stock> {
        /*let stock = unsafe {
            let x = mem::transmute::<>
        } */
        None
    }
}