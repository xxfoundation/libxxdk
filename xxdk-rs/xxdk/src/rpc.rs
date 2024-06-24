//! RPC XXDK functions.
//!
//! This module provides safe Rust wrappers around the raw FFI bindings in `xxdk-sys`.

use libc::*;
use xxdk_sys::*;

use crate::base::callbacks::{RpcResponse, RpcServerRequestHandler};
use crate::base::CMix;
use crate::util::*;

pub fn send(
    net: &CMix,
    recipient: &[u8],
    pubkey: &[u8],
    request: &[u8],
) -> Result<RpcResponse, String> {
    unsafe {
        let cmix_rpc_send_return { r0, r1 } = cmix_rpc_send(
            net.cmix_instance,
            bytes_as_go_slice(recipient),
            bytes_as_go_slice(pubkey),
            bytes_as_go_slice(request),
        );
        return go_error_into_result(
            || {
                return RpcResponse {
                    instance_id: r0,
                    response_fn: None,
                    error_fn: None,
                };
            },
            r1,
        );
    }
}

pub fn generate_reception_id(net: &CMix) -> Result<Vec<u8>, String> {
    unsafe {
        let cmix_rpc_generate_reception_id_return { r0, r1 } =
            cmix_rpc_generate_reception_id(net.cmix_instance);
        go_error_into_result(|| c_byte_slice_into_vec(r0), r1)
    }
}

pub fn generate_random_key(net: &CMix) -> Result<Vec<u8>, String> {
    unsafe {
        let cmix_rpc_generate_random_key_return { r0, r1 } =
            cmix_rpc_generate_random_key(net.cmix_instance);
        go_error_into_result(|| c_byte_slice_into_vec(r0), r1)
    }
}

pub fn derive_public_key(private_key: &[u8]) -> Result<Vec<u8>, String> {
    unsafe {
        let prk = bytes_as_go_slice(&private_key);
        let cmix_rpc_derive_public_key_return { r0, r1 } = cmix_rpc_derive_public_key(prk);
        go_error_into_result(|| c_byte_slice_into_vec(r0), r1)
    }
}

pub trait ServerCallback {
    fn serve_req(&self, sender_id: Vec<u8>, request: Vec<u8>) -> Vec<u8>;
}

pub struct Server {
    pub(crate) instance_id: i32,
    #[allow(dead_code)]
    pub(crate) cb: *mut RpcServerRequestHandler,
}

pub fn new_server<T: ServerCallback + 'static>(
    net: &CMix,
    request_callback: T,
    reception_id: Vec<u8>,
    private_key: Vec<u8>,
) -> Result<Server, String> {
    // This is absolutely unsafe to leave mutable without synchronization; I don't think it *needs*
    // to be mutable though. Investigate later
    let srh = Box::new(RpcServerRequestHandler {
        request_fn: Box::new(move |sender_id: Vec<u8>, request: Vec<u8>| -> Vec<u8> {
            tracing::debug!("inside RpceServerRequestHandler closure");
            return request_callback.serve_req(sender_id, request);
        }),
        name: String::from("new_server"),
    });
    let cb = Box::into_raw(srh);
    unsafe {
        tracing::debug!("new_server cb name: {}", (*cb).name);
        let cb_obj = cb as *const _ as *const c_void as usize;
        tracing::debug!("new_server cb_obj {:#x}", cb_obj as usize);
        let cmix_rpc_new_server_return { r0, r1 } = cmix_rpc_new_server(
            net.cmix_instance,
            cb_obj,
            bytes_as_go_slice(&reception_id),
            bytes_as_go_slice(&private_key),
        );
        go_error_into_result(
            || Server {
                instance_id: r0,
                cb,
            },
            r1,
        )
    }
}

pub fn load_server<T: ServerCallback + 'static>(
    net: &CMix,
    request_callback: T,
) -> Result<Server, String> {
    let srh = Box::new(RpcServerRequestHandler {
        request_fn: Box::new(move |sender_id: Vec<u8>, request: Vec<u8>| -> Vec<u8> {
            tracing::debug!("inside RpceServerRequestHandler closure");
            return request_callback.serve_req(sender_id, request);
        }),
        name: String::from("load_server"),
    });
    let cb = Box::into_raw(srh);
    unsafe {
        tracing::debug!("load_server cb name: {}", (*cb).name);
        let cb_obj = cb as *const _ as *const c_void as usize;
        tracing::debug!("load_server cb_obj {:#x}", cb_obj);
        let cmix_rpc_load_server_return { r0, r1 } =
            cmix_rpc_load_server(net.cmix_instance, cb_obj);
        go_error_into_result(
            || Server {
                instance_id: r0,
                cb,
            },
            r1,
        )
    }
}

impl Server {
    pub fn start(&self) {
        unsafe {
            cmix_rpc_server_start(self.instance_id);
        }
    }

    pub fn stop(&self) {
        unsafe {
            cmix_rpc_server_stop(self.instance_id);
        }
    }
}
