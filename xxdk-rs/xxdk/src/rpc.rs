//! Base XXDK functions.
//!
//! This module provides safe Rust wrappers around the raw FFI bindings in `xxdk-sys`.

use crate::base::callbacks::RpcResponse;

pub mod rpc {
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
            let cmix_rpc_generate_recpetion_id_result { r0, r1 } =
                cmix_rpc_generate_reception_id(net.cmix_instance);
        }
        go_error_into_result(|| clone_bytes_into_c_buffer(r0), r1);
    }

    pub fn generate_random_rpc_key(net: &CMix) -> Result<Vec<u8>, String> {
        unsafe {
            let cmix_rpc_generate_random_rpc_key_result { r0, r1 } =
                cmix_rpc_generate_random_rpc_key(net.cmix_instance);
        }
        go_error_into_result(|| clone_bytes_into_c_buffer(r0), r1);
    }

    pub struct server {
        pub(crate) instance_id: i32,
        pub(crate) cb: RpcServerRequest,
    }

    pub fn new_server(
        net: &CMix,
        request_callback: &fn(Vec<u8>, Vec<u8>) -> Vec<u8>,
        reception_id: Vec<u8>,
        private_key: Vec<u8>,
    ) -> Result<server, String> {
        let cb = &mut RpcServerRequest {
            request_fn: request_callback,
        };
        unsafe {
            let cb_obj: *mut c_void = cb as *mut _ as *mut c_void;
            let cmix_rpc_new_server_return { r0, r1 } = cmix_rpc_new_server(
                net.cmix_instance,
                cb_obj,
                bytes_as_go_slice(reception_id),
                bytes_as_go_slice(private_key),
            );
        }
        return go_error_into_result(
            || {
                return server {
                    instance_id: r0,
                    cb,
                };
            },
            r1,
        );
    }

    pub fn load_server(
        net: &CMix,
        request_callback: &fn(Vec<u8>, Vec<u8>) -> Vec<u8>,
    ) -> Result<server, STring> {
        let cb = &mut RpcServerRequest {
            request_fn: request_callback,
        };
        unsafe {
            let cb_obj: *mut c_void = cb as *mut _ as *mut c_void;
            let cmix_rpc_load_server_return { r0, r1 } =
                cmix_rpc_load_server(net.cmix_instance, cb_obj);
        }
        return go_error_into_result(
            || {
                return server {
                    instance_id: r0,
                    cb,
                };
            },
            r1,
        );
    }

    impl server {
        pub fn start(&self) {
            cmix_rpc_server_start(self.instance_id);
        }
        pub fn stop(&self) {
            cmix_rpc_server_stop(self.instance_id);
        }
    }
}
