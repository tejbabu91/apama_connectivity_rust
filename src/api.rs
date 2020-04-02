#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

pub mod ctypes;

pub mod public_api {
    use super::ctypes;
    use libc;
    use std::cmp::Eq;
    use std::collections::HashMap;
    use std::ffi::CStr;
    use std::fmt;
    use std::hash::{Hash, Hasher};

    pub struct TransportConstructorParameters {
        chainId: String,
        pluginName: String,
        config: HashMap<Data, Data>,
        _connectivityManager: *mut libc::c_void,
        _chain: *mut libc::c_void,
    }

    impl TransportConstructorParameters {
        pub fn new(
            name: *const ::std::os::raw::c_char,
            chainId: *const ::std::os::raw::c_char,
            config: ctypes::sag_underlying_data_t,
            connectivityManager: *mut libc::c_void,
            chain: *mut libc::c_void,
        ) -> TransportConstructorParameters {
            if let Data::Map(configMap) = super::data_conversion::c_to_rust_data(&config) {
                TransportConstructorParameters {
                    chainId: unsafe { CStr::from_ptr(chainId).to_string_lossy().into_owned() },
                    pluginName: unsafe { CStr::from_ptr(name).to_string_lossy().into_owned() },
                    config: configMap,
                    _connectivityManager: connectivityManager,
                    _chain: chain,
                }
            } else {
                panic!("config must be a map");
            }
        }

        pub fn getConfig(&self) -> &HashMap<Data, Data> {
            return &self.config;
        }
        pub fn getPluginName(&self) -> &str {
            return &self.pluginName;
        }
        pub fn getChainId(&self) -> &str {
            return &self.chainId;
        }
    }

    #[derive(Copy, Clone)]
    struct RemoteHostSide {
        host_plugin: ctypes::sag_plugin_t,
        send_fn: ctypes::sag_send_fn_t,
    }

    pub struct HostSide {
        host: std::cell::RefCell<RemoteHostSide>,
    }

    impl HostSide {
        pub fn sendMessageTowardsHost(&self, msg: Message) {
            let host = self.host.borrow();
            unsafe {
                let m = super::data_conversion::rust_to_c_msg(&msg);
                host.send_fn.unwrap()(host.host_plugin.clone(), m, m.offset(1));
                // TODO: Do we need to manually free the 'm' here?
            }
        }
        pub fn new() -> HostSide {
            HostSide {
                host: std::cell::RefCell::new(RemoteHostSide {
                    host_plugin: ctypes::sag_plugin_t {
                        r#plugin: std::ptr::null_mut(),
                    },
                    send_fn: Option::None,
                }),
            }
        }

        pub fn update(&self, host_plugin: ctypes::sag_plugin_t, send_fn: ctypes::sag_send_fn_t) {
            let mut host = self.host.borrow_mut();
            host.host_plugin = host_plugin;
            host.send_fn = send_fn;
        }
    }

    pub trait Transport {
        fn start(&self);
        fn shutdown(&self);
        fn hostReady(&self);
        fn deliverMessageTowardsTransport(&self, msg: Message);
        fn getHostSide(&self) -> &HostSide;
        fn getParams(&self) -> &TransportConstructorParameters;
        fn new(h: HostSide, params: TransportConstructorParameters) -> Box<dyn Transport>
        where
            Self: Sized;
    }

    #[repr(C)]
    pub struct WrappedTransport {
        pub transport: *mut dyn Transport,
    }

    impl std::ops::Drop for WrappedTransport {
        fn drop(&mut self) {
            unsafe {
                // Take the ownership back for the Transport object so that it gets dropped at the end of this scope.
                Box::from_raw(self.transport);
            }
        }
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
        None,
    }

    // TODO: this seems weird - if (?) we think we should implement Display, we shouldn't
    // do it by just returning the Debug format (using ?), that's just misleading
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
                _ => 0.hash(state), // TODO: use of _ like this is unsafe, we should be explicit
            };
        }
    }

    #[derive(Debug)]
    pub struct Message {
        pub payload: Data,
        pub metadata: HashMap<Data, Data>,
    }
}

pub mod plugin_impl_fn {
    use super::ctypes;
    use super::public_api::*;

    impl ctypes::sag_plugin_t {
        fn transport(&self) -> &dyn Transport {
            unsafe {
                let wt = self.r#plugin as *mut WrappedTransport;
                &*((*wt).transport)
            }
        }
    }

    pub fn rs_plugin_create_transport(transport: Box<dyn Transport>) -> ctypes::sag_plugin_t {
        let wt = Box::new(WrappedTransport{transport: Box::into_raw(transport)});
        let p  = ctypes::sag_plugin_t { r#plugin: Box::into_raw(wt) as *mut libc::c_void };
        p
    }
    pub fn rs_plugin_destroy_impl(plug: &ctypes::sag_plugin_t) -> ctypes::sag_error_t {
        unsafe {
            let wt = plug.r#plugin as *mut WrappedTransport;
            // Take the ownership back so that it gets destroyed at the end of the scope.
            Box::from_raw(wt);
        }
        ctypes::sag_error_t_SAG_ERROR_OK
    }

    pub fn rs_plugin_start_impl(p: &ctypes::sag_plugin_t) -> ctypes::sag_error_t {
        p.transport().start();
        ctypes::sag_error_t_SAG_ERROR_OK
    }

    pub fn rs_plugin_shutdown_impl(p: &ctypes::sag_plugin_t) -> ctypes::sag_error_t {
        p.transport().shutdown();
        ctypes::sag_error_t_SAG_ERROR_OK
    }

    pub fn rs_plugin_hostReady_impl(p: &ctypes::sag_plugin_t) -> ctypes::sag_error_t {
        p.transport().hostReady();
        ctypes::sag_error_t_SAG_ERROR_OK
    }

    pub fn rs_plugin_setNextTowardsHost_impl(
        this_plugin: &ctypes::sag_plugin_t,
        host_plugin: ctypes::sag_plugin_t,
        send_fn: ctypes::sag_send_fn_t,
    ) -> ctypes::sag_error_t {
        let host = this_plugin.transport().getHostSide();
        host.update(host_plugin, send_fn);
        ctypes::sag_error_t_SAG_ERROR_OK
    }

    pub extern "C" fn rs_plugin_sendBatchTowardsTransport_impl(
        plug: &ctypes::sag_plugin_t,
        start: *mut ctypes::sag_underlying_message_t,
        end: *mut ctypes::sag_underlying_message_t,
    ) -> ctypes::sag_error_t {
        unsafe {
            let mut i = 0;
            loop {
                let p = start.offset(i);
                if p == end {
                    break;
                }
                let msg = super::data_conversion::c_to_rust_msg(&*p);
                plug.transport().deliverMessageTowardsTransport(msg);
                i += 1;
            }
        }
        ctypes::sag_error_t_SAG_ERROR_OK
    }
}
pub mod data_conversion {
    use super::ctypes::*;
    use super::public_api::*;
    use std::collections::HashMap;
    use std::ffi::{CStr, CString};

    /** C++ functions to create C++ message object from Rust. */
    extern "C" {
        fn create_cpp_data_t_empty() -> *mut sag_underlying_data_t;
        fn create_cpp_data_t_bool(val: bool) -> *mut sag_underlying_data_t;
        fn create_cpp_data_t_int64(val: i64) -> *mut sag_underlying_data_t;
        fn create_cpp_data_t_double(val: f64) -> *mut sag_underlying_data_t;
        fn create_cpp_data_t_string(s: *const int_fast8_t) -> *mut sag_underlying_data_t;
        fn create_cpp_data_t_buffer(
            buf: *const uint_fast8_t,
            size_t: uint_least64_t,
        ) -> *mut sag_underlying_data_t;
        fn create_cpp_list_t_with_capacity(capacity: i64) -> *mut sag_underlying_vector_t;
        fn append_to_list_t(l: *mut sag_underlying_vector_t, d: *mut sag_underlying_data_t);
        fn create_cpp_data_t_list_t(
            val: *mut sag_underlying_vector_t,
        ) -> *mut sag_underlying_data_t;
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
        // unsafe {
        // let mut tag = sag_data_tag_SAG_DATA_EMPTY;
        // let mut val = sag_underlying_data_t__bindgen_ty_1 { boolean: true };
        match data {
            Data::None => unsafe { create_cpp_data_t_empty() },
            Data::Boolean(v) => {
                // tag = sag_data_tag_SAG_DATA_BOOLEAN;
                // val.boolean = *v;
                unsafe { create_cpp_data_t_bool(*v) }
            }
            Data::Integer(v) => {
                // tag = sag_data_tag_SAG_DATA_INTEGER;
                // val.integer = *v;
                unsafe { create_cpp_data_t_int64(*v) }
            }
            Data::Float(v) => {
                // tag = sag_data_tag_SAG_DATA_DOUBLE;
                // val.fp = *v;
                unsafe { create_cpp_data_t_double(*v) }
            }
            Data::String(v) => {
                // tag = sag_data_tag_SAG_DATA_STRING;
                // val.string = CString::new(v.as_str()).unwrap().into_raw();
                // return Box::into_raw(Box::new(sag_underlying_data_t {
                //     __bindgen_anon_1: val,
                //     tag: tag,
                // }));
                let cstr = CString::new(v.as_str()).unwrap();
                unsafe { create_cpp_data_t_string(cstr.as_ptr()) }
            }
            Data::List(v) => {
                // tag = sag_data_tag_SAG_DATA_LIST;
                // // sag_underlying_vector_table_t
                // // sag_underlying_vector
                // let mut vv: Vec<sag_underlying_data_t> = Vec::with_capacity(v.len());
                // for e in v {
                //     vv.push(rust_to_c_data(e));
                // }
                // // let x= v.as_mut_ptr();
                // let y: [sag_underlying_data_t; 1usize] = vv.as_mut_ptr();
                unsafe {
                    let l = create_cpp_list_t_with_capacity(v.len() as i64);
                    let cpp_vals: Vec<_> = v.iter().map(|d| rust_to_c_data(d)).collect();
                    for cpp_val in cpp_vals {
                        append_to_list_t(l, cpp_val);
                    }
                    create_cpp_data_t_list_t(l)
                }
            }
            Data::Map(v) => {
                // tag = sag_data_tag_SAG_DATA_MAP;
                unsafe {
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
                }
            }
            Data::Buffer(v) => {
                let size = v.len();
                unsafe { create_cpp_data_t_buffer(v.as_ptr(), size as u64) }
            } // _ => {
              //     // tag = sag_data_tag_SAG_DATA_EMPTY;
              //     unsafe { create_cpp_data_t_empty() }
              // }
        }
        // }
    }
}

#[cfg(test)]
mod test {
    use super::public_api::*;
    use std::collections::HashMap;
    #[test]
    fn test_data_hash_map() {
        let mut h: HashMap<Data, Data> = HashMap::new();
        h.insert(
            Data::String("Hello K".to_string()),
            Data::String("Hello V".to_string()),
        );
        h.insert(Data::Integer(42), Data::Integer(24));
        h.insert(Data::Float(4.2), Data::Float(2.4));
        h.insert(Data::Boolean(true), Data::Boolean(false));
        // Key and value are list
        h.insert(
            Data::List(vec![
                Data::String("k".to_string()),
                Data::Integer(42),
                Data::Float(4.2),
                Data::Boolean(true),
            ]),
            Data::List(vec![
                Data::String("v".to_string()),
                Data::Integer(24),
                Data::Float(2.4),
                Data::Boolean(false),
            ]),
        );

        // key and value are map
        let mut k1: HashMap<Data, Data> = HashMap::new();
        let mut k2: HashMap<Data, Data> = HashMap::new();
        k1.insert(Data::Integer(12), Data::Float(1.2));
        k2.insert(Data::Float(4.2), Data::Integer(42));
        h.insert(Data::Map(k1), Data::Integer(33));
        h.insert(Data::Map(k2), Data::Integer(43));

        assert_eq!(
            Data::String("Hello V".to_string()),
            h[&Data::String("Hello K".to_string())]
        );

        assert_eq!(Some(&Data::Integer(24)), h.get(&Data::Integer(42)));

        assert_eq!(Some(&Data::Float(2.4)), h.get(&Data::Float(4.2)));

        assert_eq!(Some(&Data::Boolean(false)), h.get(&Data::Boolean(true)));

        assert_eq!(Some(&Data::Boolean(false)), h.get(&Data::Boolean(true)));

        assert_eq!(
            Some(&Data::List(vec![
                Data::String("v".to_string()),
                Data::Integer(24),
                Data::Float(2.4),
                Data::Boolean(false)
            ])),
            h.get(&Data::List(vec![
                Data::String("k".to_string()),
                Data::Integer(42),
                Data::Float(4.2),
                Data::Boolean(true)
            ]))
        );

        // Check that we can retrieve value from map if key itself is a map
        let mut k1: HashMap<Data, Data> = HashMap::new();
        let mut k2: HashMap<Data, Data> = HashMap::new();
        k1.insert(Data::Integer(12), Data::Float(1.2));
        k2.insert(Data::Float(4.2), Data::Integer(42));

        assert_eq!(Some(&Data::Integer(33)), h.get(&Data::Map(k1)));

        assert_eq!(Some(&Data::Integer(43)), h.get(&Data::Map(k2)));
    }
}
