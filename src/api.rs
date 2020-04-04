#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

pub mod ctypes;
pub mod data_conversion;
pub mod plugin_impl_fn;

pub mod public_api {
    use super::ctypes;
    use libc;
    use std::cmp::Eq;
    use std::collections::HashMap;
    use std::ffi::CStr;
    use std::fmt;
    use std::hash::{Hash, Hasher};

    #[derive(Debug)]
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
        ) -> Self {
            if let Data::Map(configMap) = super::data_conversion::c_to_rust_data(&config) {
                Self {
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
        pub fn getConfigMut(&mut self) -> &mut HashMap<Data, Data> {
            return &mut self.config;
        }
        pub fn getPluginName(&self) -> &str {
            return &self.pluginName;
        }
        pub fn getChainId(&self) -> &str {
            return &self.chainId;
        }
    }

    // TODO: fix this!!!!! highly unsafe behavior
    // but no other way at the moment to pass HostSide b/w threads
    unsafe impl Send for ctypes::sag_plugin_t {}

    #[derive(Copy, Clone)]
    pub struct HostSide {
        pub host_plugin: ctypes::sag_plugin_t,
        send_fn: ctypes::sag_send_fn_t,
    }

    impl HostSide {
        pub fn sendMessageTowardsHost(&self, msg: Message) {
            unsafe {
                let m = super::data_conversion::rust_to_c_msg(&msg);
                self.send_fn.unwrap()(self.host_plugin.clone(), m, m.offset(1));
                // TODO: Do we need to manually free the 'm' here?
            }
        }
        pub fn new() -> HostSide {
            HostSide {
                host_plugin: ctypes::sag_plugin_t {
                    r#plugin: std::ptr::null_mut(),
                },
                send_fn: Option::None,
            }
        }

        pub fn update(
            &mut self,
            host_plugin: ctypes::sag_plugin_t,
            send_fn: ctypes::sag_send_fn_t,
        ) {
            self.host_plugin = host_plugin;
            self.send_fn = send_fn;
        }
    }

    pub trait Transport {
        fn start(&mut self);
        fn shutdown(&mut self);
        fn hostReady(&mut self);
        fn deliverMessageTowardsTransport(&mut self, msg: Message);
        fn getHostSide(&mut self) -> &mut HostSide;
        fn getParams(&mut self) -> &mut TransportConstructorParameters;
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

    /// Data is a Rust enum for the various kinds of data that can be passed in connectivity messages
    /// or used to configure plug-ins.   
    #[derive(Debug, PartialEq, Clone)]
    pub enum Data {
        Boolean(bool),
        Integer(i64),
        Float(f64),

        /// An owned and dynamically-sized String.
        ///
        /// Use this type for your strings unless you want to reference a static string literal
        /// in which case StaticStr will be a better choice.
        ///
        ///
        String(String),

        /// An immutable reference to a static string (or slice of one), which is a convenient  
        /// way to reference a static string literal. In future versions of the API this may be more efficient (reduced copies).
        ///
        /// This is similar to the CONST_STRING type in the C++ data_t.
        //StaticStr(&'static str), // TODO: is this still worth having, given it's no more efficient?
        List(Vec<Data>),
        Map(HashMap<Data, Data>),

        /// An untyped byte buffer.
        Buffer(Vec<u8>),

        /// Equivalent to the concept of empty in the C++ API or null in the Java API.
        None,
    }

    #[macro_export]
    macro_rules! DATA_GETTER {
        ($name:ident, $variant:ident, $the_type:ty) => {
            $crate::paste::item! {
                pub fn [<as_$name>](&self) -> &$the_type {
                    match self {
                        Data::$variant(v) => v,
                        _ => panic!("Not of type $variant on {:?}", self)
                    }
                }
                pub fn [<as_opt_$name>](&self) -> Option<&$the_type> {
                    match self {
                        Data::$variant(v) => Some(v),
                        _ => None,
                    }
                }
                pub fn [<is_$name>](&self) -> bool {
                    match self {
                        Data::$variant(_) => true,
                        _ => false,
                    }
                }
                pub fn [<to_$name>](&self) -> $the_type {
                    match self {
                        Data::$variant(v) => v.clone(),
                        _ => panic!("Not of type $variant on {:?}", self)
                    }
                }
                pub fn [<into_$name>](self) -> $the_type {
                    match self {
                        Data::$variant(v) => v,
                        _ => panic!("Not of type $variant on {:?}", self)
                    }
                }
            }
        };
    }

    impl Data {
        DATA_GETTER!(bool, Boolean, bool);
        DATA_GETTER!(int, Integer, i64);
        DATA_GETTER!(float, Float, f64);
        DATA_GETTER!(string, String, String);
        DATA_GETTER!(list, List, Vec<Data>);
        DATA_GETTER!(map, Map, HashMap<Data,Data>);
        DATA_GETTER!(buffer, Buffer, Vec<u8>);
        pub fn is_none(&self) -> bool {
            match self {
                Data::None => true,
                _ => false,
            }
        }
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
                //StaticStr(v) => v.hash(state),
                List(v) => v.hash(state),
                Buffer(v) => v.hash(state),
                Float(v) => v.to_bits().hash(state),
                Map(v) => v.len().hash(state), // using size is not great but better than nothing
                None => 0.hash(state),
            };
        }
    }

    #[derive(Debug)]
    pub struct Message {
        pub payload: Data,
        pub metadata: HashMap<Data, Data>,
    }

    // ========= Codec Support ==============
    pub struct CodecConstructorParameters {
        chainId: String,
        pluginName: String,
        config: HashMap<Data, Data>,
        _connectivityManager: *mut libc::c_void,
        _chain: *mut libc::c_void,
    }

    impl CodecConstructorParameters {
        pub fn new(
            name: *const ::std::os::raw::c_char,
            chainId: *const ::std::os::raw::c_char,
            config: ctypes::sag_underlying_data_t,
            connectivityManager: *mut libc::c_void,
            chain: *mut libc::c_void,
        ) -> Self {
            if let Data::Map(configMap) = super::data_conversion::c_to_rust_data(&config) {
                Self {
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
        pub fn getConfigMut(&mut self) -> &mut HashMap<Data, Data> {
            return &mut self.config;
        }
        pub fn getPluginName(&self) -> &str {
            return &self.pluginName;
        }
        pub fn getChainId(&self) -> &str {
            return &self.chainId;
        }
    }

    pub struct TransportSide {
        next_plugin: ctypes::sag_plugin_t,
        send_fn: ctypes::sag_send_fn_t,
    }
    impl TransportSide {
        pub fn sendMessageTowardsTransport(&self, msg: Message) {
            unsafe {
                let m = super::data_conversion::rust_to_c_msg(&msg);
                self.send_fn.unwrap()(self.next_plugin.clone(), m, m.offset(1));
            }
        }
        pub fn new() -> TransportSide {
            TransportSide {
                next_plugin: ctypes::sag_plugin_t {
                    r#plugin: std::ptr::null_mut(),
                },
                send_fn: Option::None,
            }
        }

        pub fn update(
            &mut self,
            next_plugin: ctypes::sag_plugin_t,
            send_fn: ctypes::sag_send_fn_t,
        ) {
            self.next_plugin = next_plugin;
            self.send_fn = send_fn;
        }
    }

    pub trait Codec {
        fn start(&mut self);
        fn shutdown(&mut self);
        fn hostReady(&mut self);

        fn deliverMessageTowardsTransport(&mut self, msg: Message);
        fn deliverMessageTowardsHost(&mut self, msg: Message);

        fn getParams(&mut self) -> &mut CodecConstructorParameters;
        fn getHostSide(&mut self) -> &mut HostSide;
        fn getTransportSide(&mut self) -> &mut TransportSide;

        fn new(
            host: HostSide,
            transportSide: TransportSide,
            params: CodecConstructorParameters,
        ) -> Box<dyn Codec>
        where
            Self: Sized;
    }
    #[repr(C)]
    pub struct WrappedCodec {
        pub codec: *mut dyn Codec,
    }

    impl std::ops::Drop for WrappedCodec {
        fn drop(&mut self) {
            unsafe {
                // Take the ownership back for the Codec object so that it gets dropped at the end of this scope.
                Box::from_raw(self.codec);
            }
        }
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
