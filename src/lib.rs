extern crate libc;
mod ctypes;
use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::c_char;
use crate::ctypes::*;
use std::fmt::{self, Debug, Display};

#[no_mangle]
pub extern fn add(first: i32, second: i32) -> i32 {
    println!("Inside rust: {} + {}", first, second);
    first + second
}

pub trait Transport {
    fn start(&self);
    fn get_data(&self) -> i64;
}

#[repr(C)]
pub struct WrappedTransport {
    pub transport: *mut Transport
}

#[derive(Debug)]
pub enum Data {
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    List(Vec<Data>),
    Map(HashMap<String, Data>),
    Buffer(Vec<u8>),
    None
}

impl fmt::Display for Data {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}
#[derive(Debug)]
pub struct Message {
    pub payload: Data,
    pub metadata: HashMap<String,Data>
}

#[no_mangle]
pub extern fn rust_create_transport() -> *mut WrappedTransport {
    println!("Inside create_transport");
    let mut t = create_transport();
    let mut wt = Box::new(WrappedTransport{transport: Box::into_raw(t)});
    return Box::into_raw(wt);
}

#[no_mangle]
pub extern fn call_back_from_c(t: *mut WrappedTransport){
    unsafe {
        println!("call_back_from_c_with_rust_ptr: {:p}", t);
        println!("call_back_from_c_with_rust_ptr value: {}", (*((*t).transport)).get_data());
    }
    //let mut t = Box::new(MyTransport{data: 42});
    //return &mut *t;
}

pub struct MyData {
    a: i64,
    b: i64
}

#[no_mangle]
pub extern fn send_data_towards_transport(t: *mut MyData){
    unsafe {
        println!("send_data_towards_transport: {:p}", t);
        println!("send_data_towards_transport: {}, {}", (*t).a, (*t).b);
    }
    //let mut t = Box::new(MyTransport{data: 42});
    //return &mut *t;
}


#[no_mangle]
pub extern fn send_msg_towards_transport(t: *mut sag_underlying_message_t){
    unsafe {
        println!("received_msg_in_rust_transport: {:?}, {:p}", t, t);
        //println!("send_data_towards_transport: {}, {}", (*t).a, (*t).b);
        let t = &*t;
        //println!("received_msg_in_rust_transport: tag={:?}, {:?}", t.payload.tag, CStr::from_ptr(t.payload.__bindgen_anon_1.string));
        
        let msg = c_to_rust_msg(&*t);

    println!("The msg: {:?}", msg);

    }
    //let mut t = Box::new(MyTransport{data: 42});
    //return &mut *t;
}

pub fn c_to_rust_msg(t: &sag_underlying_message_t) -> Message {
    Message {payload: c_to_rust_data(&t.payload), metadata: HashMap::new()}
}
pub fn c_to_rust_data(t: &sag_underlying_data_t) -> Data {
    unsafe {
        let tag = t.tag;
        let val = t.__bindgen_anon_1;
        match (tag) {
            sag_data_tag_SAG_DATA_EMPTY => Data::None,
            sag_data_tag_SAG_DATA_BOOLEAN => Data::Boolean(val.boolean),
            sag_data_tag_SAG_DATA_DOUBLE => Data::Float(val.fp),
            sag_data_tag_SAG_DATA_INTEGER => Data::Integer(val.integer),
            sag_data_tag_SAG_DATA_DECIMAL => Data::None,
            sag_data_tag_SAG_DATA_STRING => Data::String(CStr::from_ptr(val.string).to_string_lossy().into_owned()),
            sag_data_tag_SAG_DATA_LIST => {
                let val = &*(val.list.table);
                let mut v: Vec<Data> = Vec::with_capacity(val.count as usize);
                for x in 0..val.count {
                    // Need to use get_unchecked because C defined data as array of size 1
                    v.push(c_to_rust_data(&val.data.get_unchecked(x as usize)));
                }
                Data::List(v)
            },
            sag_data_tag_SAG_DATA_MAP => {
                let val = &*(val.map.table);
                let mut map: HashMap<String,Data> = HashMap::with_capacity(val.capacity as usize);
                for i in 0..val.capacity {
                    let entry = val.table.get_unchecked(i as usize);
                    if entry.hash <= 0 {
                        continue; // hole
                    }
                    let key = c_to_rust_data(&entry.key);
                    let value = c_to_rust_data(&entry.value);
                    // convert key into string if not a string
                    let key = match(key) {
                        Data::String(s) => s,
                        _ => key.to_string()
                    };
                    map.insert(key, value);
                }
                Data::Map(map)
            },
            sag_data_tag_SAG_DATA_BUFFER => Data::None,
            sag_data_tag_SAG_DATA_CUSTOM => Data::None,
            _ => Data::None
        }
    }
}

// ======================================== User Code =================
pub struct MyTransport {
    data: i64
}

impl Transport for MyTransport {
    fn start(&self) {
        println!("MyStransport started with {}", self.data);
    }
    fn get_data(&self) -> i64 {
        self.data
    }
}

pub fn create_transport() -> Box<Transport> {
    Box::new(MyTransport{data: 43})
}


#[cfg(test)]
mod tests {
    
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
        let x :int_least64_t = 123;
    }
}
