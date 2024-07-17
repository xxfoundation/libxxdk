//! Safe wrappers around the FFI bindings to the RPC API.

use std::pin::Pin;

use libc::*;
use xxdk_sys::*;

use crate::util::*;

use super::*;

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
        go_error_into_result(
            || RpcResponse {
                instance_id: r0,
                response_fn: None,
                error_fn: None,
            },
            r1,
        )
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
        let prk = bytes_as_go_slice(private_key);
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
    pub(crate) cb: Pin<Box<RpcServerRequestHandler>>,
}

impl CMix {
    pub fn new_rpc_server<T: ServerCallback + 'static>(
        &self,
        request_callback: T,
        reception_id: Vec<u8>,
        private_key: Vec<u8>,
    ) -> Result<Server, String> {
        let cb = Box::pin(RpcServerRequestHandler {
            request_fn: Box::new(move |sender_id: Vec<u8>, request: Vec<u8>| -> Vec<u8> {
                tracing::debug!("inside RpceServerRequestHandler closure");
                request_callback.serve_req(sender_id, request)
            }),
        });
        unsafe {
            let cb_obj = &*cb as *const _;
            tracing::debug!("new_server cb_obj {:p}", cb_obj);
            let cmix_rpc_new_server_return { r0, r1 } = cmix_rpc_new_server(
                self.cmix_instance,
                cb_obj as usize,
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

    pub fn load_rpc_server<T: ServerCallback + 'static>(
        &self,
        request_callback: T,
    ) -> Result<Server, String> {
        let cb = Box::pin(RpcServerRequestHandler {
            request_fn: Box::new(move |sender_id: Vec<u8>, request: Vec<u8>| -> Vec<u8> {
                tracing::debug!("inside RpceServerRequestHandler closure");
                request_callback.serve_req(sender_id, request)
            }),
        });
        unsafe {
            let cb_obj = &*cb as *const _;
            tracing::debug!("load_server cb_obj {:p}", cb_obj);
            let cmix_rpc_load_server_return { r0, r1 } =
                cmix_rpc_load_server(self.cmix_instance, cb_obj as usize);
            go_error_into_result(
                || Server {
                    instance_id: r0,
                    cb,
                },
                r1,
            )
        }
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

unsafe impl Send for Server {}

// RPC Callback functions

pub struct RpcResponse {
    pub(crate) instance_id: i32,
    pub response_fn: Option<Box<dyn Fn(Vec<u8>)>>,
    pub error_fn: Option<Box<dyn Fn(Vec<u8>)>>,
}

// FIXME: This may need a rework.. based on the server side of RPC it
// is better when the pointer is heap allocated, and this will
// typically be stack allocated which can cause problems. For now, we
// leave debug info in that lets us figure this out.
impl RpcResponse {
    pub fn callback(&mut self, response_fn: Box<dyn Fn(Vec<u8>)>, err_fn: Box<dyn Fn(Vec<u8>)>) {
        self.response_fn = Some(response_fn);
        self.error_fn = Some(err_fn);
        let ptr = self as *mut _ as *mut c_void as usize;
        tracing::trace!("callback conversion {:#x}", ptr);
        unsafe { cmix_rpc_send_callback(self.instance_id, ptr) }
    }
    pub fn wait(&self) {
        unsafe {
            cmix_rpc_send_wait(self.instance_id);
        }
    }
}

extern "C" fn cmix_rpc_send_response_cb(
    target: *mut c_void,
    response: *mut c_void,
    response_len: c_int,
) {
    unsafe {
        tracing::trace!(
            "cmix_rpc_send_response_cb conversion {:#x}",
            target as usize
        );
        let rpc_obj = &mut *(target as *mut RpcResponse);
        let r = response as *const u8;
        let rs = response_len as usize;
        let response = clone_bytes_from_raw_parts(r, rs);
        let rfn = rpc_obj.response_fn.as_ref().unwrap();
        rfn(response);
    }
}

extern "C" fn cmix_rpc_send_error_cb(target: *mut c_void, err: *mut c_void, err_len: c_int) {
    unsafe {
        tracing::trace!("cmix_rpc_send_error_cb conversion {:#x}", target as usize);
        let rpc_obj = &mut *(target as *mut RpcResponse);
        let e = err as *const u8;
        let es = err_len as usize;
        let response = clone_bytes_from_raw_parts(e, es);
        let efn = rpc_obj.error_fn.as_ref().unwrap();
        efn(response);
    }
}

pub struct RpcServerRequestHandler {
    #[allow(clippy::type_complexity)]
    pub request_fn: Box<dyn Fn(Vec<u8>, Vec<u8>) -> Vec<u8>>,
}

unsafe impl Send for RpcServerRequestHandler {}

extern "C" fn cmix_rpc_server_cb(
    target: *mut c_void,
    sender: *mut c_void,
    sender_len: c_int,
    request: *mut c_void,
    request_len: c_int,
) -> GoByteSlice {
    unsafe {
        tracing::trace!("cmix_rpc_server_cb conversion {:#x}", target as usize);
        let rpc_obj: &mut RpcServerRequestHandler = &mut *(target as *mut RpcServerRequestHandler);
        let s = sender as *const u8;
        let ss = sender_len as usize;
        let sndr = clone_bytes_from_raw_parts(s, ss);
        let r = request as *const u8;
        let rs = request_len as usize;
        let req = clone_bytes_from_raw_parts(r, rs);
        let sfn = &rpc_obj.request_fn;
        let res = sfn(sndr, req);
        tracing::error!(
            "cmix_rpc_sevrver_cb response: {}",
            String::from_utf8_lossy(&res)
        );
        clone_bytes_into_c_buffer(&res)
    }
}

pub fn set_rpc_callbacks() {
    tracing::trace!("set_rpc_callbacks");

    unsafe {
        register_cmix_rpc_send_callbacks(
            Some(cmix_rpc_send_response_cb),
            Some(cmix_rpc_send_error_cb),
        );
        register_cmix_rpc_server_callback(Some(cmix_rpc_server_cb));
    }
}
