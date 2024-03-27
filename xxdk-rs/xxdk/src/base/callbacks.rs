//! Exported callback functions for passing to the C library.

use std::collections::HashMap;
use std::os::raw::{c_char, c_int, c_long, c_void};
use std::sync::{Arc, RwLock};

use xxdk_sys::{DMReceiverRouterFunctions, GoByteSlice};

use crate::util::{
    clone_bytes_from_raw_parts, clone_bytes_into_c_buffer, clone_string_from_raw_parts,
};

#[derive(Default)]
pub struct DmCallbacksBuilder {
    receive: Option<Box<dyn DmReceiveCallback>>,
    receive_text: Option<Box<dyn DmReceiveTextCallback>>,
    receive_reply: Option<Box<dyn DmReceiveReplyCallback>>,
    receive_reaction: Option<Box<dyn DmReceiveReactionCallback>>,
    update_sent_status: Option<Box<dyn DmUpdateSentStatusCallback>>,
    block_sender: Option<Box<dyn DmBlockSenderCallback>>,
    unblock_sender: Option<Box<dyn DmUnblockSenderCallback>>,
    get_conversation: Option<Box<dyn DmGetConversationCallback>>,
    get_conversations: Option<Box<dyn DmGetConversationsCallback>>,
    delete_message: Option<Box<dyn DmDeleteMessageCallback>>,
    event_update: Option<Box<dyn DmEventUpdateCallback>>,
}

pub struct DmCallbacks {
    receive: Box<dyn DmReceiveCallback>,
    receive_text: Box<dyn DmReceiveTextCallback>,
    receive_reply: Box<dyn DmReceiveReplyCallback>,
    receive_reaction: Box<dyn DmReceiveReactionCallback>,
    update_sent_status: Box<dyn DmUpdateSentStatusCallback>,
    block_sender: Box<dyn DmBlockSenderCallback>,
    unblock_sender: Box<dyn DmUnblockSenderCallback>,
    get_conversation: Box<dyn DmGetConversationCallback>,
    get_conversations: Box<dyn DmGetConversationsCallback>,
    delete_message: Box<dyn DmDeleteMessageCallback>,
    event_update: Box<dyn DmEventUpdateCallback>,
}

macro_rules! builder_fns {
    ($($method_name:ident, $field_name:ident, $trait:ident;)+) => {
        $(builder_fns!($method_name, $field_name, $trait);)+
    };

    ($method_name:ident, $field_name:ident, $trait:ident) => {
        pub fn $method_name<F: $trait>(self, cb: F) -> Self {
            Self {
                $field_name: Some(Box::new(cb)),
                ..self
            }
        }
    };
}

impl DmCallbacksBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    builder_fns!(
        with_receive, receive, DmReceiveCallback;
        with_receive_text, receive_text, DmReceiveTextCallback;
        with_receive_reply, receive_reply, DmReceiveReplyCallback;
        with_receive_reaction, receive_reaction, DmReceiveReactionCallback;
        with_update_sent_status, update_sent_status, DmUpdateSentStatusCallback;
        with_block_sender, block_sender, DmBlockSenderCallback;
        with_unblock_sender, unblock_sender, DmUnblockSenderCallback;
        with_get_conversation, get_conversation, DmGetConversationCallback;
        with_get_conversations, get_conversations, DmGetConversationsCallback;
        with_delete_message, delete_message, DmDeleteMessageCallback;
        with_event_update, event_update, DmEventUpdateCallback;
    );

    pub fn build(self) -> DmCallbacks {
        if let Self {
            receive: Some(receive),
            receive_text: Some(receive_text),
            receive_reply: Some(receive_reply),
            receive_reaction: Some(receive_reaction),
            update_sent_status: Some(update_sent_status),
            block_sender: Some(block_sender),
            unblock_sender: Some(unblock_sender),
            get_conversation: Some(get_conversation),
            get_conversations: Some(get_conversations),
            delete_message: Some(delete_message),
            event_update: Some(event_update),
        } = self
        {
            DmCallbacks {
                receive,
                receive_text,
                receive_reply,
                receive_reaction,
                update_sent_status,
                block_sender,
                unblock_sender,
                get_conversation,
                get_conversations,
                delete_message,
                event_update,
            }
        } else {
            panic!("not all callbacks defined");
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DmReceiveParams<'a> {
    pub message_id: &'a [u8],
    pub nickname: &'a str,
    pub text: &'a [u8],
    pub partner_key: &'a [u8],
    pub sender_key: &'a [u8],
    pub dm_token: i32,
    pub codeset: i32,
    pub timestamp: i64,
    pub round_id: i64,
    pub message_type: i64,
    pub status: i64,
}

pub trait DmReceiveCallback: Fn(DmReceiveParams) -> i64 + Send + Sync + 'static {}
impl<F> DmReceiveCallback for F where F: Fn(DmReceiveParams) -> i64 + Send + Sync + 'static {}

#[derive(Debug, Clone, Copy)]
pub struct DmReceiveTextParams<'a> {
    pub message_id: &'a [u8],
    pub nickname: &'a str,
    pub text: &'a str,
    pub partner_key: &'a [u8],
    pub sender_key: &'a [u8],
    pub dm_token: i32,
    pub codeset: i32,
    pub timestamp: i64,
    pub round_id: i64,
    pub status: i64,
}

pub trait DmReceiveTextCallback: Fn(DmReceiveTextParams) -> i64 + Send + Sync + 'static {}
impl<F> DmReceiveTextCallback for F where F: Fn(DmReceiveTextParams) -> i64 + Send + Sync + 'static {}

#[derive(Debug, Clone, Copy)]
pub struct DmReceiveReplyParams<'a> {
    pub message_id: &'a [u8],
    pub reply_to: &'a [u8],
    pub nickname: &'a str,
    pub text: &'a str,
    pub partner_key: &'a [u8],
    pub sender_key: &'a [u8],
    pub dm_token: i32,
    pub codeset: i32,
    pub timestamp: i64,
    pub round_id: i64,
    pub status: i64,
}

pub trait DmReceiveReplyCallback: Fn(DmReceiveReplyParams) -> i64 + Send + Sync + 'static {}
impl<F> DmReceiveReplyCallback for F where F: Fn(DmReceiveReplyParams) -> i64 + Send + Sync + 'static
{}

#[derive(Debug, Clone, Copy)]
pub struct DmReceiveReactionParams<'a> {
    pub message_id: &'a [u8],
    pub reaction_to: &'a [u8],
    pub nickname: &'a str,
    pub text: &'a str,
    pub partner_key: &'a [u8],
    pub sender_key: &'a [u8],
    pub dm_token: i32,
    pub codeset: i32,
    pub timestamp: i64,
    pub round_id: i64,
    pub status: i64,
}

pub trait DmReceiveReactionCallback:
    Fn(DmReceiveReactionParams) -> i64 + Send + Sync + 'static
{
}
impl<F> DmReceiveReactionCallback for F where
    F: Fn(DmReceiveReactionParams) -> i64 + Send + Sync + 'static
{
}

#[derive(Debug, Clone, Copy)]
pub struct DmUpdateSentStatusParams<'a> {
    pub uuid: i64,
    pub message_id: &'a [u8],
    pub timestamp: i64,
    pub round_id: i64,
    pub status: i64,
}

pub trait DmUpdateSentStatusCallback: Fn(DmUpdateSentStatusParams) + Send + Sync + 'static {}
impl<F> DmUpdateSentStatusCallback for F where
    F: Fn(DmUpdateSentStatusParams) + Send + Sync + 'static
{
}

pub trait DmBlockSenderCallback: Fn(&[u8]) + Send + Sync + 'static {}
impl<F> DmBlockSenderCallback for F where F: Fn(&[u8]) + Send + Sync + 'static {}

pub trait DmUnblockSenderCallback: Fn(&[u8]) + Send + Sync + 'static {}
impl<F> DmUnblockSenderCallback for F where F: Fn(&[u8]) + Send + Sync + 'static {}

pub trait DmGetConversationCallback: Fn(&[u8]) -> Vec<u8> + Send + Sync + 'static {}
impl<F> DmGetConversationCallback for F where F: Fn(&[u8]) -> Vec<u8> + Send + Sync + 'static {}

pub trait DmGetConversationsCallback: Fn() -> Vec<u8> + Send + Sync + 'static {}
impl<F> DmGetConversationsCallback for F where F: Fn() -> Vec<u8> + Send + Sync + 'static {}

#[derive(Debug, Clone, Copy)]
pub struct DmDeleteMessageParams<'a> {
    pub message_id: &'a [u8],
    pub pubkey: &'a [u8],
}

pub trait DmDeleteMessageCallback:
    Fn(DmDeleteMessageParams) -> bool + Send + Sync + 'static
{
}
impl<F> DmDeleteMessageCallback for F where
    F: Fn(DmDeleteMessageParams) -> bool + Send + Sync + 'static
{
}

#[derive(Debug, Clone, Copy)]
pub struct DmEventUpdateParams<'a> {
    pub event_type: i64,
    pub json_data: &'a [u8],
}

pub trait DmEventUpdateCallback: Fn(DmEventUpdateParams) + Send + Sync + 'static {}
impl<F> DmEventUpdateCallback for F where F: Fn(DmEventUpdateParams) + Send + Sync + 'static {}

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
    static ref DM_INSTANCE_CALLBACKS: RwLock<HashMap<i32, Arc<DmCallbacks>>> = {
        RwLock::new(HashMap::new())
    };
}

pub fn set_callbacks(dm_instance_id: i32, callbacks: DmCallbacks) {
    DM_INSTANCE_CALLBACKS
        .write()
        .unwrap()
        .insert(dm_instance_id, Arc::new(callbacks));
}

pub fn get_callbacks(dm_instance_id: i32) -> Option<Arc<DmCallbacks>> {
    DM_INSTANCE_CALLBACKS
        .read()
        .unwrap()
        .get(&dm_instance_id)
        .cloned()
}

fn using_callbacks<F, Def, T>(dm_instance_id: c_int, default: Def, f: F) -> T
where
    F: FnOnce(&DmCallbacks) -> T,
    Def: FnOnce() -> T,
{
    if let Some(cbs) = get_callbacks(dm_instance_id as i32) {
        f(&cbs)
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

            (cbs.receive)(DmReceiveParams {
                message_id: &message_id,
                nickname: &nickname,
                text: &text,
                partner_key: &partner_key,
                sender_key: &sender_key,
                dm_token: dm_token as i32,
                codeset: codeset as i32,
                timestamp: timestamp as i64,
                round_id: round_id as i64,
                message_type: msg_type as i64,
                status: status as i64,
            }) as c_long
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

            (cbs.receive_text)(DmReceiveTextParams {
                message_id: &message_id,
                nickname: &nickname,
                text: &text,
                partner_key: &partner_key,
                sender_key: &sender_key,
                dm_token: dm_token as i32,
                codeset: codeset as i32,
                timestamp: timestamp as i64,
                round_id: round_id as i64,
                status: status as i64,
            })
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

            (cbs.receive_reply)(DmReceiveReplyParams {
                message_id: &message_id,
                reply_to: &reply_to,
                nickname: &nickname,
                text: &text,
                partner_key: &partner_key,
                sender_key: &sender_key,
                dm_token: dm_token as i32,
                codeset: codeset as i32,
                timestamp: timestamp as i64,
                round_id: round_id as i64,
                status: status as i64,
            })
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

            (cbs.receive_reaction)(DmReceiveReactionParams {
                message_id: &message_id,
                reaction_to: &reaction_to,
                nickname: &nickname,
                text: &text,
                partner_key: &partner_key,
                sender_key: &sender_key,
                dm_token: dm_token as i32,
                codeset: codeset as i32,
                timestamp: timestamp as i64,
                round_id: round_id as i64,
                status: status as i64,
            })
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
            (cbs.update_sent_status)(DmUpdateSentStatusParams {
                uuid: uuid as i64,
                message_id: &message_id,
                timestamp: timestamp as i64,
                round_id: round_id as i64,
                status: status as i64,
            })
        },
    )
}

extern "C" fn block_sender_cb(dm_instance_id: c_int, pubkey: *mut c_void, pubkey_len: c_int) {
    using_callbacks(
        dm_instance_id,
        || (),
        |cbs| unsafe {
            let pubkey = clone_bytes_from_raw_parts(pubkey as *const u8, pubkey_len as usize);
            (cbs.block_sender)(&pubkey)
        },
    )
}

extern "C" fn unblock_sender_cb(dm_instance_id: c_int, pubkey: *mut c_void, pubkey_len: c_int) {
    using_callbacks(
        dm_instance_id,
        || (),
        |cbs| unsafe {
            let pubkey = clone_bytes_from_raw_parts(pubkey as *const u8, pubkey_len as usize);
            (cbs.unblock_sender)(&pubkey)
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
            let bytes = (cbs.get_conversation)(&sender_key);
            clone_bytes_into_c_buffer(&bytes)
        },
    )
}

extern "C" fn get_conversations_cb(dm_instance_id: c_int) -> GoByteSlice {
    using_callbacks(
        dm_instance_id,
        || clone_bytes_into_c_buffer(&[]),
        |cbs| {
            let bytes = (cbs.get_conversations)();
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
            (cbs.delete_message)(DmDeleteMessageParams {
                message_id: &message_id,
                pubkey: &pubkey,
            }) as c_int
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
            (cbs.event_update)(DmEventUpdateParams {
                event_type: event_type as i64,
                json_data: &json_data,
            })
        },
    );
}
