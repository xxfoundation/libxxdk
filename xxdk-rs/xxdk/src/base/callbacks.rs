//! Exported callback functions for passing to the C library.

// We do a lot of casting e.g. `c_int` to `i32` in here, which on most systems is a no-op. Keeping
// them there for cross-platform reasons, for systems on which the C types are not the usual bit
// length.
#![allow(clippy::unnecessary_cast)]

use std::collections::HashMap;
use std::os::raw::{c_char, c_int, c_long, c_void};
use std::sync::{Arc, RwLock};

use xxdk_sys::{
    cmix_dm_set_router, cmix_rpc_send_callback, cmix_rpc_send_wait,
    register_cmix_rpc_send_callbacks, register_cmix_rpc_server_callback, DMReceiverRouterFunctions,
    GoByteSlice,
};

use crate::util::{
    clone_bytes_from_raw_parts, clone_bytes_into_c_buffer, clone_string_from_raw_parts,
};

use super::Dm;

#[allow(clippy::too_many_arguments)]
pub trait DmCallbacks: Send + Sync + 'static {
    fn receive(
        &self,
        message_id: &[u8],
        nickname: &str,
        text: &[u8],
        partner_key: &[u8],
        sender_key: &[u8],
        dm_token: i32,
        codeset: i32,
        timestamp: i64,
        round_id: i64,
        message_type: i64,
        status: i64,
    ) -> i64;

    fn receive_text(
        &self,
        message_id: &[u8],
        nickname: &str,
        text: &str,
        partner_key: &[u8],
        sender_key: &[u8],
        dm_token: i32,
        codeset: i32,
        timestamp: i64,
        round_id: i64,
        status: i64,
    ) -> i64;

    fn receive_reply(
        &self,
        message_id: &[u8],
        reply_to: &[u8],
        nickname: &str,
        text: &str,
        partner_key: &[u8],
        sender_key: &[u8],
        dm_token: i32,
        codeset: i32,
        timestamp: i64,
        round_id: i64,
        status: i64,
    ) -> i64;

    fn receive_reaction(
        &self,
        message_id: &[u8],
        reaction_to: &[u8],
        nickname: &str,
        text: &str,
        partner_key: &[u8],
        sender_key: &[u8],
        dm_token: i32,
        codeset: i32,
        timestamp: i64,
        round_id: i64,
        status: i64,
    ) -> i64;

    fn update_sent_status(
        &self,
        uuid: i64,
        message_id: &[u8],
        timestamp: i64,
        round_id: i64,
        status: i64,
    );

    fn block_sender(&self, pubkey: &[u8]);

    fn unblock_sender(&self, pubkey: &[u8]);

    fn get_conversation(&self, pubkey: &[u8]) -> Vec<u8>;

    fn get_conversations(&self) -> Vec<u8>;

    fn delete_message(&self, message_id: &[u8], pubkey: &[u8]) -> bool;

    fn event_update(&self, event_type: i64, json_data: &[u8]);
}

pub const DM_RECEIVER_ROUTER: DMReceiverRouterFunctions = DMReceiverRouterFunctions {
    receiveFn: Some(receive_cb),
    receiveTextFn: Some(receive_text_cb),
    receiveReplyFn: Some(receive_reply_cb),
    receiveReactionFn: Some(receive_reaction_cb),
    updateSentStatusFn: Some(update_sent_status_cb),
    blockSenderFn: Some(block_sender_cb),
    unblockSenderFn: Some(unblock_sender_cb),
    getConversationFn: Some(get_conversation_cb),
    getConversationsFn: Some(get_conversations_cb),
    deleteMessageFn: Some(delete_message_cb),
    eventUpdateFn: Some(event_update_cb),
};

lazy_static::lazy_static! {
    static ref DM_INSTANCE_CALLBACKS: RwLock<HashMap<i32, Arc<dyn DmCallbacks>>> = {
        unsafe {
            cmix_dm_set_router(DM_RECEIVER_ROUTER);
        }
        RwLock::new(HashMap::new())
    };
}

impl Dm {
    pub fn set_callbacks(&self, callbacks: Arc<dyn DmCallbacks>) {
        DM_INSTANCE_CALLBACKS
            .write()
            .unwrap()
            .insert(self.instance_id, callbacks);
    }

    pub fn get_callbacks(&self) -> Option<Arc<dyn DmCallbacks>> {
        DM_INSTANCE_CALLBACKS
            .read()
            .unwrap()
            .get(&self.instance_id)
            .cloned()
    }
}

fn using_callbacks<F, Def, T>(instance_id: c_int, default: Def, f: F) -> T
where
    F: FnOnce(&dyn DmCallbacks) -> T,
    Def: FnOnce() -> T,
{
    let dm = Dm {
        instance_id: instance_id as i32,
    };

    if let Some(cbs) = dm.get_callbacks() {
        f(&*cbs)
    } else {
        default()
    }
}

extern "C" fn receive_cb(
    dm_instance_id: c_int,
    message_id: *mut c_void,
    message_id_len: c_int,
    nickname: *mut c_char,
    nickname_len: c_int,
    text: *mut c_void,
    text_len: c_int,
    partner_key: *mut c_void,
    partner_key_len: c_int,
    sender_key: *mut c_void,
    sender_key_len: c_int,
    dm_token: c_int,
    codeset: c_int,
    timestamp: c_long,
    round_id: c_long,
    msg_type: c_long,
    status: c_long,
) -> c_long {
    using_callbacks(
        dm_instance_id,
        || 0,
        |cbs| unsafe {
            let message_id =
                clone_bytes_from_raw_parts(message_id as *const u8, message_id_len as usize);
            let nickname =
                clone_string_from_raw_parts(nickname as *const u8, nickname_len as usize);
            let text = clone_bytes_from_raw_parts(text as *const u8, text_len as usize);
            let partner_key =
                clone_bytes_from_raw_parts(partner_key as *const u8, partner_key_len as usize);
            let sender_key =
                clone_bytes_from_raw_parts(sender_key as *const u8, sender_key_len as usize);

            cbs.receive(
                &message_id,
                &nickname,
                &text,
                &partner_key,
                &sender_key,
                dm_token as i32,
                codeset as i32,
                timestamp as i64,
                round_id as i64,
                msg_type as i64,
                status as i64,
            ) as c_long
        },
    )
}

extern "C" fn receive_text_cb(
    dm_instance_id: c_int,
    message_id: *mut c_void,
    message_id_len: c_int,
    nickname: *mut c_char,
    nickname_len: c_int,
    text: *mut c_char,
    text_len: c_int,
    partner_key: *mut c_void,
    partner_key_len: c_int,
    sender_key: *mut c_void,
    sender_key_len: c_int,
    dm_token: c_int,
    codeset: c_int,
    timestamp: c_long,
    round_id: c_long,
    status: c_long,
) -> c_long {
    using_callbacks(
        dm_instance_id,
        || 0,
        |cbs| unsafe {
            let message_id =
                clone_bytes_from_raw_parts(message_id as *const u8, message_id_len as usize);
            let nickname =
                clone_string_from_raw_parts(nickname as *const u8, nickname_len as usize);
            let text = clone_string_from_raw_parts(text as *const u8, text_len as usize);
            let partner_key =
                clone_bytes_from_raw_parts(partner_key as *const u8, partner_key_len as usize);
            let sender_key =
                clone_bytes_from_raw_parts(sender_key as *const u8, sender_key_len as usize);

            cbs.receive_text(
                &message_id,
                &nickname,
                &text,
                &partner_key,
                &sender_key,
                dm_token as i32,
                codeset as i32,
                timestamp as i64,
                round_id as i64,
                status as i64,
            )
        },
    )
}

extern "C" fn receive_reply_cb(
    dm_instance_id: c_int,
    message_id: *mut c_void,
    message_id_len: c_int,
    reply_to: *mut c_void,
    reply_to_len: c_int,
    nickname: *mut c_char,
    nickname_len: c_int,
    text: *mut c_char,
    text_len: c_int,
    partner_key: *mut c_void,
    partner_key_len: c_int,
    sender_key: *mut c_void,
    sender_key_len: c_int,
    dm_token: c_int,
    codeset: c_int,
    timestamp: c_long,
    round_id: c_long,
    status: c_long,
) -> c_long {
    using_callbacks(
        dm_instance_id,
        || 0,
        |cbs| unsafe {
            let message_id =
                clone_bytes_from_raw_parts(message_id as *const u8, message_id_len as usize);
            let reply_to = clone_bytes_from_raw_parts(reply_to as *const u8, reply_to_len as usize);
            let nickname =
                clone_string_from_raw_parts(nickname as *const u8, nickname_len as usize);
            let text = clone_string_from_raw_parts(text as *const u8, text_len as usize);
            let partner_key =
                clone_bytes_from_raw_parts(partner_key as *const u8, partner_key_len as usize);
            let sender_key =
                clone_bytes_from_raw_parts(sender_key as *const u8, sender_key_len as usize);

            cbs.receive_reply(
                &message_id,
                &reply_to,
                &nickname,
                &text,
                &partner_key,
                &sender_key,
                dm_token as i32,
                codeset as i32,
                timestamp as i64,
                round_id as i64,
                status as i64,
            )
        },
    )
}

extern "C" fn receive_reaction_cb(
    dm_instance_id: c_int,
    message_id: *mut c_void,
    message_id_len: c_int,
    reaction_to: *mut c_void,
    reaction_to_len: c_int,
    nickname: *mut c_char,
    nickname_len: c_int,
    text: *mut c_char,
    text_len: c_int,
    partner_key: *mut c_void,
    partner_key_len: c_int,
    sender_key: *mut c_void,
    sender_key_len: c_int,
    dm_token: c_int,
    codeset: c_int,
    timestamp: c_long,
    round_id: c_long,
    status: c_long,
) -> c_long {
    using_callbacks(
        dm_instance_id,
        || 0,
        |cbs| unsafe {
            let message_id =
                clone_bytes_from_raw_parts(message_id as *const u8, message_id_len as usize);
            let reaction_to =
                clone_bytes_from_raw_parts(reaction_to as *const u8, reaction_to_len as usize);
            let nickname =
                clone_string_from_raw_parts(nickname as *const u8, nickname_len as usize);
            let text = clone_string_from_raw_parts(text as *const u8, text_len as usize);
            let partner_key =
                clone_bytes_from_raw_parts(partner_key as *const u8, partner_key_len as usize);
            let sender_key =
                clone_bytes_from_raw_parts(sender_key as *const u8, sender_key_len as usize);

            cbs.receive_reaction(
                &message_id,
                &reaction_to,
                &nickname,
                &text,
                &partner_key,
                &sender_key,
                dm_token as i32,
                codeset as i32,
                timestamp as i64,
                round_id as i64,
                status as i64,
            )
        },
    )
}

extern "C" fn update_sent_status_cb(
    dm_instance_id: c_int,
    uuid: c_long,
    message_id: *mut c_void,
    message_id_len: c_int,
    timestamp: c_long,
    round_id: c_long,
    status: c_long,
) {
    using_callbacks(
        dm_instance_id,
        || (),
        |cbs| unsafe {
            let message_id =
                clone_bytes_from_raw_parts(message_id as *const u8, message_id_len as usize);
            cbs.update_sent_status(
                uuid as i64,
                &message_id,
                timestamp as i64,
                round_id as i64,
                status as i64,
            )
        },
    )
}

extern "C" fn block_sender_cb(dm_instance_id: c_int, pubkey: *mut c_void, pubkey_len: c_int) {
    using_callbacks(
        dm_instance_id,
        || (),
        |cbs| unsafe {
            let pubkey = clone_bytes_from_raw_parts(pubkey as *const u8, pubkey_len as usize);
            cbs.block_sender(&pubkey)
        },
    )
}

extern "C" fn unblock_sender_cb(dm_instance_id: c_int, pubkey: *mut c_void, pubkey_len: c_int) {
    using_callbacks(
        dm_instance_id,
        || (),
        |cbs| unsafe {
            let pubkey = clone_bytes_from_raw_parts(pubkey as *const u8, pubkey_len as usize);
            cbs.unblock_sender(&pubkey)
        },
    )
}

extern "C" fn get_conversation_cb(
    dm_instance_id: c_int,
    sender_key: *mut c_void,
    sender_key_len: c_int,
) -> GoByteSlice {
    using_callbacks(
        dm_instance_id,
        || clone_bytes_into_c_buffer(&[]),
        |cbs| unsafe {
            let sender_key =
                clone_bytes_from_raw_parts(sender_key as *const u8, sender_key_len as usize);
            let bytes = cbs.get_conversation(&sender_key);
            clone_bytes_into_c_buffer(&bytes)
        },
    )
}

extern "C" fn get_conversations_cb(dm_instance_id: c_int) -> GoByteSlice {
    using_callbacks(
        dm_instance_id,
        || clone_bytes_into_c_buffer(&[]),
        |cbs| {
            let bytes = cbs.get_conversations();
            clone_bytes_into_c_buffer(&bytes)
        },
    )
}

extern "C" fn delete_message_cb(
    dm_instance_id: c_int,
    message_id: *mut c_void,
    message_id_len: c_int,
    pubkey: *mut c_void,
    pubkey_len: c_int,
) -> c_int {
    using_callbacks(
        dm_instance_id,
        || 0,
        |cbs| unsafe {
            let message_id =
                clone_bytes_from_raw_parts(message_id as *const u8, message_id_len as usize);
            let pubkey = clone_bytes_from_raw_parts(pubkey as *const u8, pubkey_len as usize);
            cbs.delete_message(&message_id, &pubkey) as c_int
        },
    )
}

extern "C" fn event_update_cb(
    dm_instance_id: c_int,
    event_type: c_long,
    json_data: *mut c_void,
    json_data_len: c_int,
) {
    using_callbacks(
        dm_instance_id,
        || (),
        |cbs| unsafe {
            let json_data =
                clone_bytes_from_raw_parts(json_data as *const u8, json_data_len as usize);
            cbs.event_update(event_type as i64, &json_data)
        },
    );
}

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
    pub name: String,
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
