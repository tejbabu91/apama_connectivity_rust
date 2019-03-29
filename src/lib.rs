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
use std::ptr;

macro_rules! DefineTrasport {
    ($elem:ident) => {
        #[no_mangle]
        pub extern fn rust_transport_create(owner: *mut CppOwner, config: *mut sag_underlying_map_t) -> *mut WrappedTransport {
            let config = match unsafe {config.as_ref()} {
                Some(v) => c_to_rust_map(&*v),
                None    => HashMap::<String,Data>::new()
            };
            let t = $elem::new(HostSide{owner}, config);
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
        let m = rust_to_c_msg(&msg);
        let mb = Box::into_raw(Box::new(m));
        unsafe {
            rust_send_msg_towards_host(self.owner, mb);
            let _ = Box::from_raw(mb);
        }
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
pub extern fn rust_send_msg_towards_transport(t: *mut WrappedTransport, m: *mut sag_underlying_message_t){
    unsafe {
        let msg = c_to_rust_msg(&*m);
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

#[link(name="cpplayer")]
extern {
    fn rust_send_msg_towards_host(owner: *mut CppOwner, m: *mut sag_underlying_message_t);
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
                let v = match val.list.table.as_ref() {
                    Some(val) => {
                        let mut v: Vec<Data> = Vec::with_capacity(val.count as usize);
                        for x in 0..val.count {
                            // Need to use get_unchecked because C defined data as array of size 1
                            v.push(c_to_rust_data(&val.data.get_unchecked(x as usize)));
                        }
                        v
                    },
                    None => Vec::new()
                };
                Data::List(v)
            },
            sag_data_tag_SAG_DATA_MAP => {
                Data::Map(c_to_rust_map(&val.map))
            },
            sag_data_tag_SAG_DATA_DECIMAL => Data::None,
            sag_data_tag_SAG_DATA_BUFFER => Data::None,
            sag_data_tag_SAG_DATA_CUSTOM => Data::None,
            _ => Data::None
        }
    }
}
pub fn c_to_rust_map(m: &sag_underlying_map_t) -> HashMap<String,Data> {
    unsafe {
        if let None = m.table.as_ref() {
            return HashMap::new();
        }
        let val = &*(m.table);
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
        map
    }
}

pub fn rust_to_c_msg(msg: &Message) -> sag_underlying_message_t {
    sag_underlying_message_t{
        payload: rust_to_c_data(&msg.payload),
        metadata: sag_underlying_map_t{table: ptr::null_mut()}
    }
}

#[allow(unused_variables)]
#[allow(unused_assignments)]
pub fn rust_to_c_data(data: &Data) -> sag_underlying_data_t {
    // unsafe {
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
                val.string = CString::new(v.as_str()).unwrap().into_raw();
            },
            /*
            Data::List(v) => {
                tag = sag_data_tag_SAG_DATA_LIST;
                // sag_underlying_vector_table_t
                // sag_underlying_vector
                let mut vv: Vec<sag_underlying_data_t> = Vec::with_capacity(v.len());
                for e in v {
                    vv.push(rust_to_c_data(e));
                }
                // let x= v.as_mut_ptr();
                let y: [sag_underlying_data_t; 1usize] = vv.as_mut_ptr();
            },

            Data::Map(v) => {
                tag = sag_data_tag_SAG_DATA_MAP;
            },
            */
            _ => {
                tag = sag_data_tag_SAG_DATA_EMPTY;
            }
        };
        sag_underlying_data_t{__bindgen_anon_1: val, tag:tag}
    // }
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
        let msg = Message {
            payload: Data::Integer(123),
            metadata: HashMap::new()
        };
        self.getHostSide().sendMessageTwoardsHost(msg);
        // Send some more messages back to host for testing
        let msg = Message {
            payload: Data::String("Hello from transport".to_string()),
            metadata: HashMap::new()
        };
        self.getHostSide().sendMessageTwoardsHost(msg);

        let msg = Message {
            payload: Data::Float(123.45),
            metadata: HashMap::new()
        };
        self.getHostSide().sendMessageTwoardsHost(msg);

        let msg = Message {
            payload: Data::Boolean(true),
            metadata: HashMap::new()
        };
        self.getHostSide().sendMessageTwoardsHost(msg);

        let msg = Message {
            payload: Data::None,
            metadata: HashMap::new()
        };
        self.getHostSide().sendMessageTwoardsHost(msg);
    }
    fn getHostSide(&self) -> HostSide {
        self.hostSide
    }
}

impl MyTransport {
    fn new(h: HostSide, config: HashMap<String,Data>) -> Box<Transport> {
        println!("Creating transport with config {:?}", config);
        Box::new(MyTransport{data: 43, hostSide: h})
    }
}

DefineTrasport!(MyTransport);

