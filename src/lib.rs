pub mod api;

pub use crate::api::*;

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
