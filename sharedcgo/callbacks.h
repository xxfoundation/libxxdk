/////////////////////////////////////////////////////////////////////////////
//                                                                         //
// Direct Messaging Callbacks (you must implement these)                   //
//                                                                         //
/////////////////////////////////////////////////////////////////////////////

#ifndef CALLBACKS_H
#define CALLBACKS_H

#include <stddef.h>
#include <stdint.h>

// A cMix instance ID.
typedef int Cmix;

typedef struct {
  int   len;
  void* data;
} GoByteSlice;

typedef char *GoError;
typedef void (*cmix_status_callback_fn)(int, void *);
typedef size_t (*xx_log_output_fn)(void *, const void *, size_t);

// Set and return the thread-local info string.
//
// This will lazily initialize the string for the current thread, and overwrite
// any previous contents. It may also reallocate the string's buffer, if more
// space is required for the given contents.
//
// The given len should be the byte length of the contents ignoring any
// terminating null byte. This function will add a terminating null byte to the
// thread-local string after copying the given contents.
const char *set_info_string(const void *contents, size_t new_len);

typedef long (* cmix_dm_receive_fn)(int dm_instance_id,
  void* message_id, int message_id_len,
  char* nickname, int nickname_len,
  void* text, int text_len,
  void* partnerkey, int partnerkey_len,
  void* senderkey, int senderkey_len,
  int dmToken, int codeset,
  long timestamp, long round_id, long msg_type, long status);
typedef long (* cmix_dm_receive_text_fn)(int dm_instance_id,
  void* mesage_id, int message_id_len,
  char* nickname, int nickname_len,
  char* text, int text_len,
  void* partnerkey, int partnerkey_len,
  void* senderkey, int senderkey_len,
  int dmToken, int codeset,
  long timestamp, long round_id, long status);
typedef long (* cmix_dm_receive_reply_fn)(int dm_instance_id,
  void* mesage_id, int message_id_len,
  void* reply_to, int reply_to_len,
  char* nickname, int nickname_len,
  char* text, int text_len,
  void* partnerkey, int partnerkey_len,
  void* senderkey, int senderkey_len,
  int dmToken, int codeset,
  long timestamp, long round_id, long status);
typedef long (* cmix_dm_receive_reaction_fn)(int dm_instance_id,
  void* mesage_id, int message_id_len,
  void* reaction_to, int reaction_to_len,
  char* nickname, int nickname_len,
  char* text, int text_len,
  void* partnerkey, int partnerkey_len,
  void* senderkey, int senderkey_len,
  int dmToken, int codeset,
  long timestamp, long round_id, long status);
typedef void (* cmix_dm_update_sent_status_fn)(int dm_instance_id,
  long uuid,
  void* message_id, int message_id_len, long timestamp,
  long round_id, long status);
typedef void (* cmix_dm_block_sender_fn)(int dm_instance_id,
  void* pubkey, int pubkey_len);
typedef void (* cmix_dm_unblock_sender_fn)(int dm_instance_id,
  void* pubkey, int pubkey_len);
typedef GoByteSlice (* cmix_dm_get_conversation_fn)(int dm_instance_id,
  void* senderkey, int senderkey_len);
typedef GoByteSlice (* cmix_dm_get_conversations_fn)(int dm_instance_id);
typedef int (* cmix_dm_delete_message_fn)(int dm_instance_id,
  void* message_id, int message_id_len,
  void* pubkey, int pubkey_len);
typedef void (* cmix_dm_event_update_fn)(int dm_instance_id,
  long event_type, void* json_data,
  int json_data_len);

// This struct values must be set by your program, the symbol is called
// "DMReceiverRouter"
typedef struct {
  cmix_dm_receive_fn receiveFn;
  cmix_dm_receive_text_fn receiveTextFn;
  cmix_dm_receive_reply_fn receiveReplyFn;
  cmix_dm_receive_reaction_fn receiveReactionFn;
  cmix_dm_update_sent_status_fn updateSentStatusFn;
  cmix_dm_block_sender_fn blockSenderFn;
  cmix_dm_unblock_sender_fn unblockSenderFn;
  cmix_dm_get_conversation_fn getConversationFn;
  cmix_dm_get_conversations_fn getConversationsFn;
  cmix_dm_delete_message_fn deleteMessageFn;
  cmix_dm_event_update_fn eventUpdateFn;
} DMReceiverRouterFunctions;

typedef void (* cmix_rpc_send_response_fn)(void *obj,
  void *response, int response_len);
typedef void (* cmix_rpc_send_error_fn)(void *obj,
  void *error_str, int error_str_len);
typedef GoByteSlice (* cmix_rpc_server_callback_fn)(void *obj,
  void *sender, int sender_len,
  void *request, int request_len);

#ifdef _WIN32
#define DLL_EXPORT __declspec(dllexport)
#else
#define DLL_EXPORT
#endif

#endif
