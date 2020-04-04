use super::ctypes;
use super::public_api::*;

impl ctypes::sag_plugin_t {
    fn transport(&mut self) -> &mut dyn Transport {
        unsafe {
            let wt = self.r#plugin as *mut WrappedTransport;
            &mut *((*wt).transport)
        }
    }
    fn codec(&mut self) -> &mut dyn Codec {
        unsafe {
            let wt = self.r#plugin as *mut WrappedCodec;
            &mut *((*wt).codec)
        }
    }
}

pub fn rs_plugin_create_transport(transport: Box<dyn Transport>) -> ctypes::sag_plugin_t {
    let wt = Box::new(WrappedTransport {
        transport: Box::into_raw(transport),
    });
    let p = ctypes::sag_plugin_t {
        r#plugin: Box::into_raw(wt) as *mut libc::c_void,
    };
    p
}
pub fn rs_plugin_destroy_impl(plug: &mut ctypes::sag_plugin_t) -> ctypes::sag_error_t {
    unsafe {
        let wt = plug.r#plugin as *mut WrappedTransport;
        // Take the ownership back so that it gets destroyed at the end of the scope.
        Box::from_raw(wt);
    }
    ctypes::sag_error_t_SAG_ERROR_OK
}

pub fn rs_plugin_start_impl(p: &mut ctypes::sag_plugin_t) -> ctypes::sag_error_t {
    p.transport().start();
    ctypes::sag_error_t_SAG_ERROR_OK
}

pub fn rs_plugin_shutdown_impl(p: &mut ctypes::sag_plugin_t) -> ctypes::sag_error_t {
    p.transport().shutdown();
    ctypes::sag_error_t_SAG_ERROR_OK
}

pub fn rs_plugin_hostReady_impl(p: &mut ctypes::sag_plugin_t) -> ctypes::sag_error_t {
    p.transport().hostReady();
    ctypes::sag_error_t_SAG_ERROR_OK
}

pub fn rs_plugin_setNextTowardsHost_impl(
    this_plugin: &mut ctypes::sag_plugin_t,
    host_plugin: ctypes::sag_plugin_t,
    send_fn: ctypes::sag_send_fn_t,
) -> ctypes::sag_error_t {
    let host = this_plugin.transport().getHostSide();
    host.update(host_plugin, send_fn);
    ctypes::sag_error_t_SAG_ERROR_OK
}

pub extern "C" fn rs_plugin_sendBatchTowardsTransport_impl(
    plug: &mut ctypes::sag_plugin_t,
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

pub fn rs_plugin_create_codec(codec: Box<dyn Codec>) -> ctypes::sag_plugin_t {
    let wt = Box::new(WrappedCodec {
        codec: Box::into_raw(codec),
    });
    let p = ctypes::sag_plugin_t {
        r#plugin: Box::into_raw(wt) as *mut libc::c_void,
    };
    p
}
pub fn rs_plugin_destroy_codec_impl(plug: &mut ctypes::sag_plugin_t) -> ctypes::sag_error_t {
    unsafe {
        let wt = plug.r#plugin as *mut WrappedCodec;
        // Take the ownership back so that it gets destroyed at the end of the scope.
        Box::from_raw(wt);
    }
    ctypes::sag_error_t_SAG_ERROR_OK
}

pub fn rs_plugin_start_codec_impl(p: &mut ctypes::sag_plugin_t) -> ctypes::sag_error_t {
    p.codec().start();
    ctypes::sag_error_t_SAG_ERROR_OK
}

pub fn rs_plugin_shutdown_codec_impl(p: &mut ctypes::sag_plugin_t) -> ctypes::sag_error_t {
    p.codec().shutdown();
    ctypes::sag_error_t_SAG_ERROR_OK
}

pub fn rs_plugin_hostReady_codec_impl(p: &mut ctypes::sag_plugin_t) -> ctypes::sag_error_t {
    p.codec().hostReady();
    ctypes::sag_error_t_SAG_ERROR_OK
}

pub fn rs_plugin_setNextTowardsHost_codec_impl(
    this_plugin: &mut ctypes::sag_plugin_t,
    next_plugin: ctypes::sag_plugin_t,
    send_fn: ctypes::sag_send_fn_t,
) -> ctypes::sag_error_t {
    let side = this_plugin.codec().getHostSide();
    side.update(next_plugin, send_fn);
    ctypes::sag_error_t_SAG_ERROR_OK
}

pub fn rs_plugin_setNextTowardsTransport_codec_impl(
    this_plugin: &mut ctypes::sag_plugin_t,
    next_plugin: ctypes::sag_plugin_t,
    send_fn: ctypes::sag_send_fn_t,
) -> ctypes::sag_error_t {
    let side = this_plugin.codec().getTransportSide();
    side.update(next_plugin, send_fn);
    ctypes::sag_error_t_SAG_ERROR_OK
}

pub extern "C" fn rs_plugin_sendBatchTowardsTransport_codec_impl(
    plug: &mut ctypes::sag_plugin_t,
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
            plug.codec().deliverMessageTowardsTransport(msg);
            i += 1;
        }
    }
    ctypes::sag_error_t_SAG_ERROR_OK
}

pub extern "C" fn rs_plugin_sendBatchTowardsHost_codec_impl(
    plug: &mut ctypes::sag_plugin_t,
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
            plug.codec().deliverMessageTowardsHost(msg);
            i += 1;
        }
    }
    ctypes::sag_error_t_SAG_ERROR_OK
}
