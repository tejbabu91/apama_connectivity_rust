extern crate libc;

use std::collections::HashMap;

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

enum DataType {
    Boolean(bool),
    Integer(i64),
    Double(f64),
    String(String),
    List(Vec<DataType>),
    Map(HashMap<DataType, DataType>),
    Buffer(Vec<u8>)
}

pub struct Message {
    payload: DataType,
    metadata: HashMap<DataType,DataType>
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

pub struct Data {
    a: i64,
    b: i64
}

#[no_mangle]
pub extern fn send_msg_towards_transport(t: *mut Data){
    unsafe {
        println!("send_msg_towards_transport: {:p}", t);
        println!("send_msg_towards_transport: {}, {}", (*t).a, (*t).b);
    }
    //let mut t = Box::new(MyTransport{data: 42});
    //return &mut *t;
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
    }
}
