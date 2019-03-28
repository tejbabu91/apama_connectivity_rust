extern crate libc;
mod ctypes;
use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::c_char;
use crate::ctypes::*;

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

pub enum Data {
    Boolean(bool),
    Integer(i64),
    Double(f64),
    String(String),
    List(Vec<Data>),
    Map(HashMap<Data, Data>),
    Buffer(Vec<u8>)
}

#[repr(C)]
pub struct Message {
    pub payload: Data,
    pub metadata: HashMap<Data,Data>
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
        println!("received_msg_in_rust_transport: tag={:?}, {:?}", t.payload.tag, CStr::from_ptr(t.payload.__bindgen_anon_1.string));
        
        c_to_rust_msg(&*t);


    }
    //let mut t = Box::new(MyTransport{data: 42});
    //return &mut *t;
}

pub fn c_to_rust_msg(t: &sag_underlying_message_t) {
    c_to_rust_data(&t.payload);
}
pub fn c_to_rust_data(t: &sag_underlying_data_t) {

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
