pub mod api;

pub use crate::api::*;

use crate::api::ctypes as ctypes;
use paste;

// sag_is_host_shutting_down from c_functions.hpp was commented out because of using reference parameters
// default parameter value was removed from sag_copy_custom in c_functions.hpp

pub struct MyTestTransport {
    data: i64
}

#[macro_export]
macro_rules! DEFINE_RUST_TRANSPORT {
    ($elem:ident) => {
        paste::item! {
            #[no_mangle]
            pub extern fn [<sag_plugin_api_version_$elem>](p: ctypes::sag_plugin_t) -> ctypes::__uint64_t {
                4
            }

            #[no_mangle]
            pub extern fn [<sag_create_plugin_with_params_$elem>](name : *const ::std::os::raw::c_char, chainId: *const ::std::os::raw::c_char, config: ctypes::sag_underlying_data_t, _connectivityManager: *mut libc::c_void, _reserved: *mut libc::c_void) -> ctypes::sag_plugin_t {
                println!("GYS: sag_create_plugin_with_params_MyNewTestTransport - start");
                let t = Box::new(MyTestTransport{data: 42});

                let p  = ctypes::sag_plugin_t { r#plugin: Box::into_raw(t) as *mut libc::c_void };

                println!("GYS: sag_create_plugin_with_params_MyNewTestTransport - end");

                p
            }
            #[no_mangle]
            pub extern fn [<sag_destroy_plugin_$elem>](p: ctypes::sag_plugin_t) -> ctypes::sag_error_t {
                println!("GYS: sag_destroy_plugin_MyNewTestTransport");

                ctypes::sag_error_t_SAG_ERROR_OK
            }
            #[no_mangle]
            pub extern fn [<sag_plugin_start_$elem>](p: ctypes::sag_plugin_t) -> ctypes::sag_error_t {
                unsafe {
                    let pp = Box::from_raw(p.r#plugin as *mut MyTestTransport) ;
                    println!("GYS: sag_plugin_start_MyNewTestTransport: {}", pp.data);
                }

                ctypes::sag_error_t_SAG_ERROR_OK
            }
            #[no_mangle]
            pub extern fn [<sag_plugin_shutdown_$elem>](p: ctypes::sag_plugin_t) -> ctypes::sag_error_t {
                println!("GYS: sag_plugin_shutdown_MyNewTestTransport");

                ctypes::sag_error_t_SAG_ERROR_OK
            }
            #[no_mangle]
            pub extern fn [<sag_plugin_hostReady_$elem>](p: ctypes::sag_plugin_t) -> ctypes::sag_error_t {
                println!("GYS: sag_plugin_hostReady_MyNewTestTransport");

                ctypes::sag_error_t_SAG_ERROR_OK
            }
            #[no_mangle]
            pub extern fn [<sag_plugin_setNextTowardsHost_$elem>](p: ctypes::sag_plugin_t, q: ctypes::sag_plugin_t, send_fn: ctypes::sag_set_next_fn_t)  -> ctypes::sag_error_t {
                println!("GYS: sag_plugin_setNextTowardsHost_MyNewTestTransport");
                // TODO - what do I need to implement here.
                ctypes::sag_error_t_SAG_ERROR_OK
            }

            #[no_mangle]
            pub extern fn [<sag_plugin_sendBatchTowardsTransport_$elem>](p: ctypes::sag_plugin_t, start: *mut ctypes::sag_underlying_message_t, end: *mut ctypes::sag_underlying_message_t,) -> ctypes::sag_error_t {
                println!("GYS: sag_plugin_sendBatchTowardsTransport_MyNewTestTransport");
                // TODO - what do I need to implement here.
                ctypes::sag_error_t_SAG_ERROR_OK
            }
        }
    }
}

DEFINE_RUST_TRANSPORT!(MyTestTransport);

#[macro_export]
macro_rules! DefineTransport {
    ($elem:ident) => {
        #[no_mangle]
        pub extern fn rust_transport_create(owner: *mut rust_ap_connectivity::CppOwner, config: *mut rust_ap_connectivity::api::ctypes::sag_underlying_map_t) -> *mut rust_ap_connectivity::WrappedTransport {
            use std::collections::HashMap;
			use rust_ap_connectivity::{Data, HostSide};
            let config = match unsafe {config.as_ref()} {
                Some(v) => c_to_rust_map(&*v),
                None    => HashMap::<Data,Data>::new()
            };
            let t = $elem::new(HostSide::new(owner), config);
            // TODO: We are leaking the transport object at the moment as
            // we are not doing manual cleanup of raw pointers in the C++
            // destructor.
            let wt = Box::new(rust_ap_connectivity::WrappedTransport{transport: Box::into_raw(t)});
            return Box::into_raw(wt);
        }
    };
}
