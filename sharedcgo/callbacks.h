/////////////////////////////////////////////////////////////////////////////
//                                                                         //
// Direct Messaging Callbacks (you must implement these)                   //
//                                                                         //
/////////////////////////////////////////////////////////////////////////////

#ifndef CALLBACKS_H
#define CALLBACKS_H

typedef long (* cmix_dm_receive_cb)(int dm_instance_id,
   void* message_id, int message_id_len,
   char* nickname, int nickname_len,
   void* text, int text_len,
   void* partnerkey, int partnerkey_len,
   void* senderkey, int senderkey_len,
   int dmToken, int codeset,
   long timestamp, long round_id, long msg_type, long status);
typedef long (* cmix_dm_receive_text_cb)(int dm_instance_id,
   void* mesage_id, int message_id_len,
   char* nickname, int nickname_len,
   char* text, int text_len,
   void* partnerkey, int partnerkey_len,
   void* senderkey, int senderkey_len,
   int dmToken, int codeset,
   long timestamp, long round_id, long status);
typedef long (* cmix_dm_receive_reply_cb)(int dm_instance_id,
   void* mesage_id, int message_id_len,
   void* reply_to, int reply_to_len,
   char* nickname, int nickname_len,
   char* text, int text_len,
   void* partnerkey, int partnerkey_len,
   void* senderkey, int senderkey_len,
   int dmToken, int codeset,
   long timestamp, long round_id, long status);
typedef long (* cmix_dm_receive_reaction_cb)(int dm_instance_id,
   void* mesage_id, int message_id_len,
   void* reaction_to, int reaction_to_len,
   char* nickname, int nickname_len,
   char* text, int text_len,
   void* partnerkey, int partnerkey_len,
   void* senderkey, int senderkey_len,
   int dmToken, int codeset,
   long timestamp, long round_id, long status);
typedef void (* cmix_dm_update_sent_status_cb)(int dm_instance_id,
   long uuid,
   void* message_id, int message_id_len, long timestamp,
   long round_id, long status);

// This struct values must be set by your program, the symbol is called
// "DMReceiverCallbacks"
typedef struct {
   cmix_dm_receive_cb receiveFn;
   cmix_dm_receive_text_cb receiveTextFn;
   cmix_dm_receive_reply_cb receiveReplyFn;
   cmix_dm_receive_reaction_cb receiveReactionFn;
   cmix_dm_update_sent_status_cb updateSentStatusFn;
} DMReceiverCallbackFunctions;


#endif
