////////////////////////////////////////////////////////////////////////////////
// Copyright © 2022 xx foundation                                             //
//                                                                            //
// Use of this source code is governed by a license that can be found in the  //
// LICENSE file.                                                              //
////////////////////////////////////////////////////////////////////////////////

package main

// #include "callbacks.h"
// #cgo CFLAGS: -I .
//
// DMReceiverCallbackFunctions DMReceiverCallbacks;
//
// /////////////////////////////////////////////////////////////////////////////
// //                                                                         //
// // Direct Messaging Go Callbacks                                           //
// //                                                                         //
// // These are are called by go to call the functions defined in the         //
// // DMReceiverCallbacks struct. Do not modify. They have to be defined      //
// // here because of cgo rules.                                              //
// /////////////////////////////////////////////////////////////////////////////
//
// long cmix_dm_receive(int dm_instance_id,
//    void* message_id, int message_id_len,
//    char* nickname, int nickname_len,
//    void* text, int text_len,
//    void* pubkey, int pubkey_len,
//    int dmToken, int codeset,
//    long timestamp, long round_id, long msg_type, long status) {
//    return DMReceiverCallbacks.receiveFn(dm_instance_id,
//       message_id, message_id_len, nickname, nickname_len,
//       text, text_len, pubkey, pubkey_len, dmToken, codeset,
//       timestamp, round_id, msg_type, status);
// }
// long cmix_dm_receive_text(int dm_instance_id,
//    void* message_id, int message_id_len,
//    char* nickname, int nickname_len,
//    char* text, int text_len,
//    void* pubkey, int pubkey_len,
//    int dmToken, int codeset,
//    long timestamp, long round_id, long status) {
//    return DMReceiverCallbacks.receiveTextFn(dm_instance_id,
//       message_id, message_id_len, nickname, nickname_len,
//       text, text_len, pubkey, pubkey_len, dmToken, codeset,
//       timestamp, round_id, status);
// }
// long cmix_dm_receive_reply(int dm_instance_id,
//    void* message_id, int message_id_len,
//    void* reply_to, int reply_to_len,
//    char* nickname, int nickname_len,
//    char* text, int text_len,
//    void* pubkey, int pubkey_len,
//    int dmToken, int codeset,
//    long timestamp, long round_id, long status) {
//    return DMReceiverCallbacks.receiveReplyFn(dm_instance_id,
//       message_id, message_id_len, reply_to, reply_to_len,
//       nickname, nickname_len, text, text_len, pubkey, pubkey_len,
//       dmToken, codeset, timestamp, round_id, status);
// }
// long cmix_dm_receive_reaction(int dm_instance_id,
//    void* message_id, int message_id_len,
//    void* reaction_to, int reaction_to_len,
//    char* nickname, int nickname_len,
//    char* text, int text_len,
//    void* pubkey, int pubkey_len,
//    int dmToken, int codeset,
//    long timestamp, long round_id, long status) {
//    return DMReceiverCallbacks.receiveReactionFn(dm_instance_id,
//       message_id, message_id_len, reaction_to, reaction_to_len,
//       nickname, nickname_len, text, text_len, pubkey, pubkey_len,
//       dmToken, codeset, timestamp, round_id, status);
// }
// extern void cmix_dm_update_sent_status(int dm_instance_id,
//    long uuid,
//    void* message_id, int message_id_len, long timestamp,
//    long round_id, long status) {
//    DMReceiverCallbacks.updateSentStatusFn(dm_instance_id, uuid,
//        message_id, message_id_len, timestamp, round_id, status);
// }
import "C"
