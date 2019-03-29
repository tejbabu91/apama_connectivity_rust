#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_imports)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

extern crate libc;

pub mod ctypes;

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use ctypes::*;
use std::fmt::{self, Debug, Display};
use std::ptr;
use std::cmp::{PartialEq, Eq};
use std::hash::{Hash, Hasher};

pub enum CppOwner {}

// Copy should be cheap as it contains only c++ pointer.
#[derive(Copy, Clone)]
pub struct HostSide {
    pub owner: *mut CppOwner
}
impl HostSide {
    pub fn sendMessageTwoardsHost(&self, msg: Message) {
        println!("Called sendMessageTwoardsHost: {:?}", msg);
        let m = rust_to_c_msg(&msg);
        let mb = Box::into_raw(Box::new(m));
        unsafe {
            rust_send_msg_towards_host(self.owner, mb);
            let _ = Box::from_raw(mb);
        }
    }
    pub fn new(owner: *mut CppOwner) -> HostSide {
        HostSide{owner}
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

#[derive(Debug, PartialEq)]
pub enum Data {
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    List(Vec<Data>),
    Map(HashMap<Data, Data>),
    Buffer(Vec<u8>),
    None
}

impl fmt::Display for Data {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Eq for Data {}
impl Hash for Data {
    fn hash<H: Hasher>(&self, state: &mut H) {
        use Data::*;
        match &self {
          Boolean(v) => v.hash(state),
          Integer(v) => v.hash(state),
          String(v) => v.hash(state),
          List(v) => v.hash(state),
          Buffer(v) => v.hash(state),
          Float(v) => v.to_bits().hash(state),
          // Map gets 0 hashcode - inefficient but should produce correct values
          _ => 0.hash(state)
        };
    }
}

#[derive(Debug)]
pub struct Message {
    pub payload: Data,
    pub metadata: HashMap<Data,Data>
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


#[no_mangle]
pub extern fn rust_transport_destroy(t: *mut WrappedTransport) {
    unsafe {
        // take ownership back so that rust can destroy it.
        let bw = Box::from_raw(t);
        let _bt = Box::from_raw(bw.transport);
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
pub fn c_to_rust_map(m: &sag_underlying_map_t) -> HashMap<Data,Data> {
    unsafe {
        if let None = m.table.as_ref() {
            return HashMap::new();
        }
        let val = &*(m.table);
        let mut map: HashMap<Data,Data> = HashMap::with_capacity(val.capacity as usize);
        for i in 0..val.capacity {
            let entry = val.table.get_unchecked(i as usize);
            if entry.hash <= 0 {
                continue; // hole
            }
            let key = c_to_rust_data(&entry.key);
            let value = c_to_rust_data(&entry.value);
            // convert key into string if not a string
            // let key = match key {
            //     Data::String(s) => s,
            //     _ => key.to_string()
            // };
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



#[test]
fn test_data_hash_map() {
    let mut h: HashMap<Data,Data> = HashMap::new();
    h.insert(Data::String("Hello K".to_string()),Data::String("Hello V".to_string()));
    h.insert(Data::Integer(42),Data::Integer(24));
    h.insert(Data::Float(4.2),Data::Float(2.4));
    h.insert(Data::Boolean(true),Data::Boolean(false));
    // Key and value are list
    h.insert(
        Data::List(vec![
            Data::String("k".to_string()), Data::Integer(42), Data::Float(4.2), Data::Boolean(true)]),
        Data::List(vec![
            Data::String("v".to_string()), Data::Integer(24), Data::Float(2.4), Data::Boolean(false)]));

    // key and value are map
    let mut k1: HashMap<Data,Data> = HashMap::new();
    let mut k2: HashMap<Data,Data> = HashMap::new();
    k1.insert(Data::Integer(12), Data::Float(1.2));
    k2.insert(Data::Float(4.2), Data::Integer(42));
    
    h.insert(Data::Map(k1), Data::Integer(33));
    h.insert(Data::Map(k2), Data::Integer(43));

    assert_eq!(
        Data::String("Hello V".to_string()),
        h[&Data::String("Hello K".to_string())]);

    assert_eq!(
        Some(&Data::Integer(24)),
        h.get(&Data::Integer(42)));

    assert_eq!(
        Some(&Data::Float(2.4)),
        h.get(&Data::Float(4.2)));

    assert_eq!(
        Some(&Data::Boolean(false)),
        h.get(&Data::Boolean(true)));

    assert_eq!(
        Some(&Data::Boolean(false)),
        h.get(&Data::Boolean(true)));

    assert_eq!(
        Some(&Data::List(vec![
            Data::String("v".to_string()), Data::Integer(24), Data::Float(2.4), Data::Boolean(false)])),
        h.get(&Data::List(vec![
            Data::String("k".to_string()), Data::Integer(42), Data::Float(4.2), Data::Boolean(true)])));

    // Check that we can retrieve value from map if key itself is a map
    let mut k1: HashMap<Data,Data> = HashMap::new();
    let mut k2: HashMap<Data,Data> = HashMap::new();
    k1.insert(Data::Integer(12), Data::Float(1.2));
    k2.insert(Data::Float(4.2), Data::Integer(42));

    assert_eq!(
        Some(&Data::Integer(33)),
        h.get(&Data::Map(k1)));

    assert_eq!(
        Some(&Data::Integer(43)),
        h.get(&Data::Map(k2)));
}