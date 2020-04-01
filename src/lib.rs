#![allow(dead_code)]
pub mod api;

pub use crate::api::*;


// sag_is_host_shutting_down from c_functions.hpp was commented out because of using reference parameters
// default parameter value was removed from sag_copy_custom in c_functions.hpp

#[macro_export]
macro_rules! DECLARE_CONNECTIVITY_TRANSPORT {
    ($elem:ident) => {
        use paste;
        paste::item! {
            #[no_mangle]
            pub extern fn [<sag_plugin_api_version_$elem>](p: rust_ap_connectivity::api::ctypes::sag_plugin_t) -> ctypes::__uint64_t {
                4
            }

            #[no_mangle]
            pub extern fn [<sag_create_plugin_with_params_$elem>](name : *const ::std::os::raw::c_char, chainId: *const ::std::os::raw::c_char, config: rust_ap_connectivity::api::ctypes::sag_underlying_data_t, _connectivityManager: *mut libc::c_void, _reserved: *mut libc::c_void) -> ctypes::sag_plugin_t {
                println!("GYS: sag_create_plugin_with_params_MyNewTestTransport - start");
                // let t = Box::new(MyTestTransport{data: 42});
                let t = $elem::new(HostSide::new(), std::collections::HashMap::new());
                // TODO: We are leaking the transport object at the moment as
                // we are not doing manual cleanup of raw pointers in the C++
                // destructor.
                let wt = Box::new(WrappedTransport{transport: Box::into_raw(t)});
                /// return Box::into_raw(wt);


                //let p  = ctypes::sag_plugin_t { r#plugin: Box::into_raw(t) as *mut libc::c_void };
                let p  = rust_ap_connectivity::api::ctypes::sag_plugin_t { r#plugin: Box::into_raw(wt) as *mut libc::c_void };
                
                println!("GYS: sag_create_plugin_with_params_MyNewTestTransport - end");
                // TODO: We are not saving chainManager and chain instance anywhere yet?

                p
            }
            #[no_mangle]
            pub extern fn [<sag_destroy_plugin_$elem>](p: rust_ap_connectivity::api::ctypes::sag_plugin_t) -> rust_ap_connectivity::api::ctypes::sag_error_t {
                println!("GYS: sag_destroy_plugin_MyNewTestTransport");

                rust_ap_connectivity::api::ctypes::sag_error_t_SAG_ERROR_OK
            }
            #[no_mangle]
            pub extern fn [<sag_plugin_start_$elem>](p: rust_ap_connectivity::api::ctypes::sag_plugin_t) -> rust_ap_connectivity::api::ctypes::sag_error_t {
                println!("GYS: sag_plugin_start_MyNewTestTransport: - start");
                unsafe {
                    //let pp = Box::from_raw(p.r#plugin as *mut WrappedTransport) ;
                    // TODO: we are using already define function for now - we should clean it up.
                    rust_transport_start(p.r#plugin as *mut WrappedTransport);
                    // println!("GYS: sag_plugin_start_MyNewTestTransport: {}", pp.transport.start());
                }
                println!("GYS: sag_plugin_start_MyNewTestTransport: - end");
                rust_ap_connectivity::api::ctypes::sag_error_t_SAG_ERROR_OK
            }
            #[no_mangle]
            pub extern fn [<sag_plugin_shutdown_$elem>](p: rust_ap_connectivity::api::ctypes::sag_plugin_t) -> rust_ap_connectivity::api::ctypes::sag_error_t {
                println!("GYS: sag_plugin_shutdown_MyNewTestTransport: - start");
                unsafe {
                    //let pp = Box::from_raw(p.r#plugin as *mut WrappedTransport) ;
                    // TODO: we are using already define function for now - we should clean it up.
                    rust_transport_shutdown(p.r#plugin as *mut WrappedTransport);
                }
                println!("GYS: sag_plugin_shutdown_MyNewTestTransport: - end");
                rust_ap_connectivity::api::ctypes::sag_error_t_SAG_ERROR_OK
            }
            #[no_mangle]
            pub extern fn [<sag_plugin_hostReady_$elem>](p: rust_ap_connectivity::api::ctypes::sag_plugin_t) -> rust_ap_connectivity::api::ctypes::sag_error_t {
                println!("GYS: sag_plugin_hostReady_MyNewTestTransport - start");
                unsafe {
                    //let pp = Box::from_raw(p.r#plugin as *mut WrappedTransport) ;
                    // TODO: we are using already define function for now - we should clean it up.
                    rust_transport_hostReady(p.r#plugin as *mut WrappedTransport);
                }
                println!("GYS: sag_plugin_hostReady_MyNewTestTransport - end");
                rust_ap_connectivity::api::ctypes::sag_error_t_SAG_ERROR_OK
            }
            #[no_mangle]
            pub extern fn [<sag_plugin_setNextTowardsHost_$elem>](p: rust_ap_connectivity::api::ctypes::sag_plugin_t, q: rust_ap_connectivity::api::ctypes::sag_plugin_t, send_fn: rust_ap_connectivity::api::ctypes::sag_send_fn_t)  -> rust_ap_connectivity::api::ctypes::sag_error_t {
                println!("GYS: sag_plugin_setNextTowardsHost_MyNewTestTransport - start");

                unsafe {
                    let mut wt = p.r#plugin as *mut WrappedTransport;
                    let mut  host = (*((*wt).transport)).getHostSide();
                    host.update(q, send_fn);
                }
                println!("GYS: sag_plugin_setNextTowardsHost_MyNewTestTransport - end");
                rust_ap_connectivity::api::ctypes::sag_error_t_SAG_ERROR_OK
            }

            #[no_mangle]
            pub extern fn [<sag_plugin_sendBatchTowardsTransport_$elem>](plug: rust_ap_connectivity::api::ctypes::sag_plugin_t, start: *mut rust_ap_connectivity::api::ctypes::sag_underlying_message_t, end: *mut rust_ap_connectivity::api::ctypes::sag_underlying_message_t,) -> rust_ap_connectivity::api::ctypes::sag_error_t {
                println!("GYS: sag_plugin_sendBatchTowardsTransport_MyNewTestTransport");
                unsafe {
                    let mut i  = 0;
                    loop {
                        let p = start.offset(i);
                        if (p == end) {
                            break;
                        }
                        rust_send_msg_towards_transport(plug.r#plugin as *mut WrappedTransport, p);
                        
                        i += 1;
                    }
                }
                // TODO - what do I need to implement here.
                rust_ap_connectivity::api::ctypes::sag_error_t_SAG_ERROR_OK
            }
        }
    }
}
