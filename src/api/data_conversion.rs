use super::ctypes::*;
use super::public_api::*;
use std::collections::HashMap;
use std::ffi::CStr;

/** C++ functions to create C++ message object from Rust. */
extern "C" {
    fn create_cpp_data_t_empty() -> *mut sag_underlying_data_t;
    fn create_cpp_data_t_bool(val: bool) -> *mut sag_underlying_data_t;
    fn create_cpp_data_t_int64(val: i64) -> *mut sag_underlying_data_t;
    fn create_cpp_data_t_double(val: f64) -> *mut sag_underlying_data_t;
    fn create_cpp_data_t_string(s: *const int_fast8_t, len: usize) -> *mut sag_underlying_data_t;
    fn create_cpp_data_t_buffer(
        buf: *const uint_fast8_t,
        size_t: uint_least64_t,
    ) -> *mut sag_underlying_data_t;
    fn create_cpp_list_t_with_capacity(capacity: i64) -> *mut sag_underlying_vector_t;
    fn append_to_list_t(l: *mut sag_underlying_vector_t, d: *mut sag_underlying_data_t);
    fn create_cpp_data_t_list_t(val: *mut sag_underlying_vector_t) -> *mut sag_underlying_data_t;
    fn create_cpp_map_t() -> *mut sag_underlying_map_t;
    fn insert_into_map_t(
        m: *mut sag_underlying_map_t,
        key: *mut sag_underlying_data_t,
        value: *mut sag_underlying_data_t,
    );
    fn create_cpp_data_t_map_t(val: *mut sag_underlying_map_t) -> *mut sag_underlying_data_t;
    fn create_cpp_message_t(
        payload: *mut sag_underlying_data_t,
        metadata: *mut sag_underlying_map_t,
    ) -> *mut sag_underlying_message_t;
    pub fn free_cpp_message_t(m: *mut sag_underlying_message_t);
}

pub fn c_to_rust_msg(t: &sag_underlying_message_t) -> Message {
    Message {
        payload: c_to_rust_data(&t.payload),
        metadata: c_to_rust_map(&t.metadata),
    }
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
            sag_data_tag_SAG_DATA_STRING => {
                Data::String(CStr::from_ptr(val.string).to_string_lossy().into_owned())
            }
            sag_data_tag_SAG_DATA_LIST => {
                let v = match val.list.table.as_ref() {
                    Some(val) => {
                        let mut v: Vec<Data> = Vec::with_capacity(val.count as usize);
                        for x in 0..val.count {
                            // Need to use get_unchecked because C defined data as array of size 1
                            v.push(c_to_rust_data(&val.data.get_unchecked(x as usize)));
                        }
                        v
                    }
                    None => Vec::new(),
                };
                Data::List(v)
            }
            sag_data_tag_SAG_DATA_MAP => Data::Map(c_to_rust_map(&val.map)),
            sag_data_tag_SAG_DATA_DECIMAL => Data::None,
            sag_data_tag_SAG_DATA_BUFFER => match val.buffer.table.as_ref() {
                Some(x) => {
                    let bufsize = x.length as usize;
                    let mut rbuf: Vec<u8> = Vec::with_capacity(bufsize);
                    std::ptr::copy_nonoverlapping(x.data.as_ptr(), rbuf.as_mut_ptr(), bufsize);
                    Data::Buffer(rbuf)
                }
                None => Data::Buffer(Vec::new()),
            },
            sag_data_tag_SAG_DATA_CUSTOM => Data::None,
            _ => Data::None,
        }
    }
}
pub fn c_to_rust_map(m: &sag_underlying_map_t) -> HashMap<Data, Data> {
    unsafe {
        if let None = m.table.as_ref() {
            return HashMap::new();
        }
        let val = &*(m.table);
        let mut map: HashMap<Data, Data> = HashMap::with_capacity(val.capacity as usize);
        for i in 0..val.capacity {
            let entry = val.table.get_unchecked(i as usize);
            if entry.hash <= 0 {
                continue; // hole
            }
            let key = c_to_rust_data(&entry.key);
            let value = c_to_rust_data(&entry.value);
            map.insert(key, value);
        }
        map
    }
}

pub fn rust_to_c_msg(msg: &Message) -> *mut sag_underlying_message_t {
    let payload = rust_to_c_data(&msg.payload);
    unsafe {
        let cpp_metadata = create_cpp_map_t();
        for (k, v) in msg.metadata.iter() {
            let cpp_key = rust_to_c_data(k);
            let cpp_val = rust_to_c_data(v);
            insert_into_map_t(cpp_metadata, cpp_key, cpp_val);
        }
        create_cpp_message_t(payload, cpp_metadata)
    }
}

#[allow(unused_variables)]
#[allow(unused_assignments)]
pub fn rust_to_c_data(data: &Data) -> *mut sag_underlying_data_t {
    match data {
        Data::None => unsafe { create_cpp_data_t_empty() },
        Data::Boolean(v) => unsafe { create_cpp_data_t_bool(*v) },
        Data::Integer(v) => unsafe { create_cpp_data_t_int64(*v) },
        Data::Float(v) => unsafe { create_cpp_data_t_double(*v) },
        Data::String(v) => {
            // TODO: maybe we should do some validation to check that the Rust string doesn't contain nulls (perhaps CStr can help us with that)

            // pass the length explicitly because in Rust, string buffers are not null-terminated; plus, it's good practice
            unsafe { create_cpp_data_t_string(v.as_ptr() as *const i8, v.len()) }
        }
        /*Data::StaticStr(v) => {
            // unfortunately can't use the zero-copy data_t CONST_STRING constructor as data_t assumes its strings are null-terminated which isn't the case for Rust strings
            unsafe { create_cpp_data_t_string(v.as_ptr() as *const i8, v.len()) }
        }*/
        Data::List(v) => unsafe {
            let l = create_cpp_list_t_with_capacity(v.len() as i64);
            let cpp_vals: Vec<_> = v.iter().map(|d| rust_to_c_data(d)).collect();
            for cpp_val in cpp_vals {
                append_to_list_t(l, cpp_val);
            }
            create_cpp_data_t_list_t(l)
        },
        Data::Map(v) => unsafe {
            let m = create_cpp_map_t();
            let cpp_vals: Vec<_> = v
                .iter()
                .map(|(k, v)| (rust_to_c_data(k), rust_to_c_data(v)))
                .collect();
            for cpp_val in cpp_vals {
                let (key, val) = cpp_val;
                insert_into_map_t(m, key, val);
            }
            create_cpp_data_t_map_t(m)
        },
        Data::Buffer(v) => {
            let size = v.len();
            unsafe { create_cpp_data_t_buffer(v.as_ptr(), size as u64) }
        }
    }
}
