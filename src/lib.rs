#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_imports)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

extern crate libc;

mod ctypes;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use crate::ctypes::*;
use std::fmt::{self, Debug, Display};


macro_rules! DefineTrasport {
    ($elem:ident) => {
        #[no_mangle]
        pub extern fn rust_transport_create(owner: *mut CppOwner) -> *mut WrappedTransport {
            println!("Inside create_transport");
            let t = $elem::new(HostSide{owner});
            // TODO: We are leaking the transport object at the moment as
            // we are not doing manual cleanup of raw pointers in the C++
            // destructor.
            let wt = Box::new(WrappedTransport{transport: Box::into_raw(t)});
            return Box::into_raw(wt);
        }
    };
}

pub enum CppOwner {}

// Copy should be cheap as it contains only c++ pointer.
#[derive(Copy, Clone)]
pub struct HostSide {
    owner: *mut CppOwner
}
impl HostSide {
    fn sendMessageTwoardsHost(&self, msg: Message) {
        println!("Called sendMessageTwoardsHost: {:?}", msg);
    }
}

pub trait Transport {
    fn start(&self);
    fn shutdown(&self);
    fn hostReady(&self);
    fn deliverMessageTowardsTransport(&self, msg: Message);
    fn getHostSide(&self) -> HostSide;
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
    }
}
#[derive(Debug)]
pub struct Message {
    pub payload: Data,
    pub metadata: HashMap<String,Data>
}

#[no_mangle]
pub extern fn rust_transport_send_msg_towards(t: *mut WrappedTransport, m: *mut sag_underlying_message_t){
    unsafe {
        println!("received_msg_in_rust_transport: {:?}, {:p}", m, m);
        let m = &*m;
        let msg = c_to_rust_msg(m);
        println!("The msg: {:?}", msg);
        (*((*t).transport)).deliverMessageTowardsTransport(msg);
    }
}

#[no_mangle]
pub extern fn rust_transport_start(t: *mut WrappedTransport) {
    unsafe {
        (*((*t).transport)).start();
    }
}

#[no_mangle]
pub extern fn rust_transport_shutdown(t: *mut WrappedTransport) {
    unsafe {
        (*((*t).transport)).shutdown();
    }
}

#[no_mangle]
pub extern fn rust_transport_hostReady(t: *mut WrappedTransport) {
    unsafe {
        (*((*t).transport)).hostReady();
    }
}

pub fn c_to_rust_msg(t: &sag_underlying_message_t) -> Message {
    Message {payload: c_to_rust_data(&t.payload), metadata: HashMap::new()}
}
pub fn c_to_rust_data(t: &sag_underlying_data_t) -> Data {
    unsafe {
        let tag = t.tag;
        let val = t.__bindgen_anon_1;
        match tag {
            sag_data_tag_SAG_DATA_EMPTY => Data::None,
            sag_data_tag_SAG_DATA_BOOLEAN => Data::Boolean(val.boolean),
            sag_data_tag_SAG_DATA_DOUBLE => Data::Float(val.fp),
            sag_data_tag_SAG_DATA_INTEGER => Data::Integer(val.integer),
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
                    let key = match key {
                        Data::String(s) => s,
                        _ => key.to_string()
                    };
                    map.insert(key, value);
                }
                Data::Map(map)
            },
            sag_data_tag_SAG_DATA_DECIMAL => Data::None,
            sag_data_tag_SAG_DATA_BUFFER => Data::None,
            sag_data_tag_SAG_DATA_CUSTOM => Data::None,
            _ => Data::None
        }
    }
}
#[allow(unused_variables)]
#[allow(unused_assignments)]
pub fn rust_to_c_data(data: &Data) {
    unsafe {
        let mut tag = sag_data_tag_SAG_DATA_EMPTY;
        let mut val = sag_underlying_data_t__bindgen_ty_1 {boolean: true};
        match data {
            Data::None => {
                tag = sag_data_tag_SAG_DATA_EMPTY;
            },
            Data::Boolean(v) => {
                tag = sag_data_tag_SAG_DATA_BOOLEAN;
                val.boolean = *v;
            }
            Data::Integer(v) => {
                tag = sag_data_tag_SAG_DATA_INTEGER;
                val.integer = *v;
            },
            Data::Float(v) => {
                tag = sag_data_tag_SAG_DATA_DOUBLE;
                val.fp = *v;
            },
            Data::String(v) => {
                tag = sag_data_tag_SAG_DATA_STRING;
                let x = CString::new(v.as_str()).unwrap();
                // TODO: Convert string to raw::c_char
            },
            Data::List(v) => {
                tag = sag_data_tag_SAG_DATA_LIST;
            },

            Data::Map(v) => {
                tag = sag_data_tag_SAG_DATA_MAP;
            },
            _ => {
                tag = sag_data_tag_SAG_DATA_EMPTY;
            }
        };
    }
}

// ======================================== User Code =================
pub struct MyTransport {
    data: i64,
    hostSide: HostSide
}

impl Transport for MyTransport {
    fn start(&self) {
        println!("MyTransport started with {}", self.data);
    }
    fn shutdown(&self) {
        println!("MyTransport shutdown done");
    }
    fn hostReady(&self) {
        println!("MyTransport handled hostReady");
    }
    fn deliverMessageTowardsTransport(&self, msg: Message) {
        println!("MyTransport received message from host: {:?}", msg);
        self.getHostSide().sendMessageTwoardsHost(msg);
    }
    fn getHostSide(&self) -> HostSide {
        self.hostSide
    }
}

impl MyTransport {
    fn new(h: HostSide) -> Box<Transport> {
        Box::new(MyTransport{data: 43, hostSide: h})
    }
}

DefineTrasport!(MyTransport);

