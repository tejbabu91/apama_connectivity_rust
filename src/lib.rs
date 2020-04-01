pub mod api;
// export things in public_api modules only
pub use crate::api::public_api::*;
// re-export for the user crate
pub use libc;
pub use paste;

/*
The bindgen tool was run on sag_connectivity_c.h file and the output was saved in the api/ctypes.rs file.
Few changes were made to the c_functions.hpp header file to make bidngen work:
    - The sag_is_host_shutting_down functions was commented out because of it using reference parameter which is not valid C.
    - Default parameter value was removed from sag_copy_custom function because it is not valid C.
*/

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
            pub extern fn [<sag_create_plugin_with_params_$elem>](
                name : *const ::std::os::raw::c_char, 
                chainId: *const ::std::os::raw::c_char, 
                config: ctypes::sag_underlying_data_t,
                 _connectivityManager: *mut libc::c_void, 
                 _reserved: *mut libc::c_void
            ) -> ctypes::sag_plugin_t {
                // TODO: is this if let... else panic construct the best way to handle this? don't really like creating a new scope for something this simple
                if let Data::Map(configMap) = rust_ap_connectivity::api::data_conversion::c_to_rust_data(&config) {
                // TODO: change to using a Parameters object for the new() method like in Java and C++ for extensibility purposes
                let t = $elem::new(
                        HostSide::new(), 
                        configMap
                    );
                    let wt = Box::new(WrappedTransport{transport: Box::into_raw(t)});
                    let p  = ctypes::sag_plugin_t { r#plugin: Box::into_raw(wt) as *mut libc::c_void };
                    p
                } else {
                    panic!("config must be a map");
                }         
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
