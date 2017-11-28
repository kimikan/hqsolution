
use std::ffi::CStr;
use std::os::raw::*;


#[link(name = "ct2sdk")]
extern "C" {
    pub fn create_t2context() -> *mut c_void;

    pub fn release_t2context(_: *mut c_void);

    pub fn send_message(_: *mut c_void, msg: *mut c_void) -> i32;
    pub fn set_callback(_: *mut c_void, cb: extern fn(*mut c_void));

    pub fn t2message_create() -> *mut c_void;

    pub fn t2message_release(msg: *mut c_void);

    //REQUEST_PACKET or ANSWER_PACKET
    pub fn t2message_setpackettype(msg: *mut c_void, _: i32);

    pub fn t2message_getfunctionno(msg: *mut c_void) -> i32;

    pub fn t2message_setfunctionno(msg: *mut c_void, func: i32);

    pub fn t2message_beginpack(msg: *mut c_void);

    pub fn t2message_endpack(msg: *mut c_void);

    pub fn t2message_addfield(msg: *mut c_void, name: *const c_char);

    pub fn t2message_addchar(msg: *mut c_void, c: u8);

    pub fn t2message_addstr(msg: *mut c_void, name: *const c_char);

    pub fn t2message_addint(msg: *mut c_void, value: i64);

    pub fn msgparser_create(msg: *mut c_void, len: i32) -> *mut c_void;

    pub fn msgparser_release(msg: *mut c_void);
    pub fn msgparser_getcolcount(msg: *mut c_void) -> i32;

    pub fn msgparser_getstr(msg: *mut c_void, name: *const c_char) -> *const c_char;

    pub fn msgparser_getchar(msg: *mut c_void, name: *const c_char) -> u8;

    pub fn msgparser_getdouble(msg: *mut c_void, name: *const c_char) -> f64;

    pub fn msgparser_getint(msg: *mut c_void, name: *const c_char) -> i64;
    pub fn msgparser_getraw(msg: *mut c_void, name: *const c_char, len: *mut c_int) -> *mut c_void;
    pub fn msgparser_wasnull(msg: *mut c_void) -> i32;

    pub fn msgparser_next(msg: *mut c_void);

    pub fn msgparser_iseof(msg: *mut c_void) -> i32;

    pub fn msgparser_isempty(msg: *mut c_void) -> i32;

    pub fn msgparser_destroy(_: *mut c_void) -> *mut c_void;
}


use std::os::raw::*;

pub struct T2Context {
    _context: *mut c_void,
}

impl T2Context {
    pub fn new() -> T2Context {
        T2Context { _context: unsafe { create_t2context() } }
    }

    pub fn set_callback(&mut self, cb: extern fn(*mut c_void)) {
        unsafe {
            set_callback(self._context, cb);
        }
    }
}

impl Drop for T2Context {
    fn drop(&mut self) {
        unsafe { release_t2context(self._context) };
    }
}

pub struct T2Message {
    _message: *mut c_void,
}

impl T2Message {
    pub fn new() -> T2Message {
        T2Message {
            _message : unsafe { t2message_create() },
        }
    }

    pub fn set_packet_type(&mut self, t: i32) {
        unsafe {
            t2message_setpackettype(self._message, t);
        }
    }

    pub fn set_function_no(&mut self, func: i32) {
        unsafe {
            t2message_setfunctionno(self._message, func);
        }
    }
}

impl Drop for T2Message {
    fn drop(&mut self) {
        unsafe {
            t2message_release(self._message);
        }
    }
}

use std::ffi::CString;
fn to_char_array(s:&str)->*const c_char {
    let cs = CString::new(s).unwrap();
    let pt = cs.as_ptr();
    pt
}

pub extern fn callback(msg:*mut c_void) {
        if msg as u32 != 0 {
             unsafe {msgparser_release(msg) };
        }
}
