//! Base XXDK functions.
//!
//! This module provides safe Rust wrappers around the raw FFI bindings in `xxdk-sys`.

use libc::*;
use xxdk_sys::*;

use crate::base::callbacks::{RpcResponse, RpcServerRequest};
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
        unsafe {
            let cmix_rpc_derive_public_key_return { r0, r1 } = cmix_rpc_derive_public_key(prk);
            go_error_into_result(|| c_byte_slice_into_vec(r0), r1)
        }
    }
}

pub struct Server {
    pub(crate) instance_id: i32,
    #[allow(dead_code)]
    pub(crate) cb: RpcServerRequest,
}

pub fn new_server(
    net: &CMix,
    request_callback: fn(Vec<u8>, Vec<u8>) -> Vec<u8>,
    reception_id: Vec<u8>,
    private_key: Vec<u8>,
) -> Result<Server, String> {
    // This is absolutely unsafe to leave mutable without synchronization; I don't think it *needs*
    // to be mutable though. Investigate later
    let mut cb = RpcServerRequest {
        request_fn: request_callback,
    };
    unsafe {
        let cb_obj: *mut c_void = &mut cb as *mut _ as *mut c_void;
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

pub fn load_server(
    net: &CMix,
    request_callback: fn(Vec<u8>, Vec<u8>) -> Vec<u8>,
) -> Result<Server, String> {
    let mut cb = RpcServerRequest {
        request_fn: request_callback,
    };
    unsafe {
        let cb_obj: *mut c_void = &mut cb as *mut _ as *mut c_void;
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
