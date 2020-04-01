#![allow(dead_code)]
pub mod api;

pub use crate::api::public_api::*; // export things in public_api modules only
pub use paste; // re-export for the user crate
pub use libc; // re-export for the user crate


// sag_is_host_shutting_down from c_functions.hpp was commented out because of using reference parameters
// default parameter value was removed from sag_copy_custom in c_functions.hpp

#[macro_export]
macro_rules! DECLARE_CONNECTIVITY_TRANSPORT {
    ($elem:ident) => {
        use rust_ap_connectivity::api::ctypes;
        use paste;
        use rust_ap_connectivity::api::public_api::{WrappedTransport};
        paste::item! {
            #[no_mangle]
            pub extern fn [<sag_plugin_api_version_$elem>](p: ctypes::sag_plugin_t) -> ctypes::__uint64_t {
                4
            }

            #[no_mangle]
            pub extern fn [<sag_create_plugin_with_params_$elem>](name : *const ::std::os::raw::c_char, chainId: *const ::std::os::raw::c_char, config: ctypes::sag_underlying_data_t, _connectivityManager: *mut libc::c_void, _reserved: *mut libc::c_void) -> ctypes::sag_plugin_t {
                let t = $elem::new(HostSide::new(), std::collections::HashMap::new());
                let wt = Box::new(WrappedTransport{transport: Box::into_raw(t)});
                let p  = ctypes::sag_plugin_t { r#plugin: Box::into_raw(wt) as *mut libc::c_void };
                p
            }

            #[no_mangle]
            pub extern fn [<sag_destroy_plugin_$elem>](p: ctypes::sag_plugin_t) -> ctypes::sag_error_t {
                rust_ap_connectivity::api::plugin_impl_fn::rs_plugin_destroy_impl(&p)
            }
            
            #[no_mangle]
            pub extern fn [<sag_plugin_start_$elem>](p: ctypes::sag_plugin_t) -> ctypes::sag_error_t {
                rust_ap_connectivity::api::plugin_impl_fn::rs_plugin_start_impl(&p)
            }
            #[no_mangle]
            pub extern fn [<sag_plugin_shutdown_$elem>](p: ctypes::sag_plugin_t) -> ctypes::sag_error_t {
                rust_ap_connectivity::api::plugin_impl_fn::rs_plugin_shutdown_impl(&p)
            }
            #[no_mangle]
            pub extern fn [<sag_plugin_hostReady_$elem>](p: ctypes::sag_plugin_t) -> ctypes::sag_error_t {
                rust_ap_connectivity::api::plugin_impl_fn::rs_plugin_hostReady_impl(&p)
            }
            #[no_mangle]
            pub extern fn [<sag_plugin_setNextTowardsHost_$elem>](p: ctypes::sag_plugin_t, host_plugin: ctypes::sag_plugin_t, send_fn: ctypes::sag_send_fn_t)  -> ctypes::sag_error_t {
                rust_ap_connectivity::api::plugin_impl_fn::rs_plugin_setNextTowardsHost_impl(&p, host_plugin, send_fn)
            }

            #[no_mangle]
            pub extern fn [<sag_plugin_sendBatchTowardsTransport_$elem>](plug: ctypes::sag_plugin_t, start: *mut ctypes::sag_underlying_message_t, end: *mut ctypes::sag_underlying_message_t,) -> ctypes::sag_error_t {
                rust_ap_connectivity::api::plugin_impl_fn::rs_plugin_sendBatchTowardsTransport_impl(&plug, start, end)
            }
        }
    }
}
