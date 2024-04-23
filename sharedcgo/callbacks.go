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
// DMReceiverRouterFunctions DMReceiverRouter;
// ChannelsRouterFunctions ChannelsRouter;
//
// /////////////////////////////////////////////////////////////////////////////
// //                                                                         //
// // Direct Messaging Go Callbacks                                           //
// //                                                                         //
// // These are are called by go to call the functions defined in the         //
// // DMReceiverRouter struct. Do not modify. They have to be defined         //
// // here because of cgo rules.                                              //
// /////////////////////////////////////////////////////////////////////////////
//
// DLL_EXPORT long cmix_dm_receive(int dm_instance_id,
//    void* message_id, int message_id_len,
//    char* nickname, int nickname_len,
//    void* text, int text_len,
//    void* partnerkey, int partnerkey_len,
//    void* senderkey, int senderkey_len,
//    int dmToken, int codeset,
//    long timestamp, long round_id, long msg_type, long status) {
//    return DMReceiverRouter.receiveFn(dm_instance_id,
//       message_id, message_id_len, nickname, nickname_len,
//       text, text_len, partnerkey, partnerkey_len, senderkey, senderkey_len,
//       dmToken, codeset, timestamp, round_id, msg_type, status);
// }
// DLL_EXPORT long cmix_dm_receive_text(int dm_instance_id,
//    void* message_id, int message_id_len,
//    char* nickname, int nickname_len,
//    char* text, int text_len,
//    void* partnerkey, int partnerkey_len,
//    void* senderkey, int senderkey_len,
//    int dmToken, int codeset,
//    long timestamp, long round_id, long status) {
//    return DMReceiverRouter.receiveTextFn(dm_instance_id,
//       message_id, message_id_len, nickname, nickname_len,
//       text, text_len, partnerkey, partnerkey_len, senderkey, senderkey_len,
//       dmToken, codeset, timestamp, round_id, status);
// }
// DLL_EXPORT long cmix_dm_receive_reply(int dm_instance_id,
//    void* message_id, int message_id_len,
//    void* reply_to, int reply_to_len,
//    char* nickname, int nickname_len,
//    char* text, int text_len,
//    void* partnerkey, int partnerkey_len,
//    void* senderkey, int senderkey_len,
//    int dmToken, int codeset,
//    long timestamp, long round_id, long status) {
//    return DMReceiverRouter.receiveReplyFn(dm_instance_id,
//       message_id, message_id_len, reply_to, reply_to_len,
//       nickname, nickname_len,
//       text, text_len, partnerkey, partnerkey_len, senderkey, senderkey_len,
//       dmToken, codeset, timestamp, round_id, status);
// }
// DLL_EXPORT long cmix_dm_receive_reaction(int dm_instance_id,
//    void* message_id, int message_id_len,
//    void* reaction_to, int reaction_to_len,
//    char* nickname, int nickname_len,
//    char* text, int text_len,
//    void* partnerkey, int partnerkey_len,
//    void* senderkey, int senderkey_len,
//    int dmToken, int codeset,
//    long timestamp, long round_id, long status) {
//    return DMReceiverRouter.receiveReactionFn(dm_instance_id,
//       message_id, message_id_len, reaction_to, reaction_to_len,
//       nickname, nickname_len,
//       text, text_len, partnerkey, partnerkey_len, senderkey, senderkey_len,
//       dmToken, codeset, timestamp, round_id, status);
// }
// DLL_EXPORT void cmix_dm_update_sent_status(int dm_instance_id,
//    long uuid,
//    void* message_id, int message_id_len, long timestamp,
//    long round_id, long status) {
//    DMReceiverRouter.updateSentStatusFn(dm_instance_id, uuid,
//        message_id, message_id_len, timestamp, round_id, status);
// }
// DLL_EXPORT void cmix_dm_block_sender(int dm_instance_id, void* pubkey,
//    int pubkey_len) {
//    DMReceiverRouter.blockSenderFn(dm_instance_id, pubkey, pubkey_len);
// }
// DLL_EXPORT void cmix_dm_unblock_sender(int dm_instance_id, void* pubkey,
//    int pubkey_len) {
//    DMReceiverRouter.unblockSenderFn(dm_instance_id, pubkey, pubkey_len);
// }
// DLL_EXPORT GoByteSlice cmix_dm_get_conversation(int dm_instance_id,
//    void* senderkey, int senderkey_len) {
//    return DMReceiverRouter.getConversationFn(dm_instance_id, senderkey,
//      senderkey_len);
// }
// DLL_EXPORT GoByteSlice cmix_dm_get_conversations(int dm_instance_id) {
//    return DMReceiverRouter.getConversationsFn(dm_instance_id);
// }
// DLL_EXPORT int cmix_dm_delete_message(int dm_instance_id,
//    void* message_id, int message_id_len,
//    void* pubkey, int pubkey_len) {
//    return DMReceiverRouter.deleteMessageFn(dm_instance_id,
//        message_id, message_id_len,
//        pubkey, pubkey_len);
// }
// DLL_EXPORT void cmix_dm_event_update(int dm_instance_id,
//    long event_type, void* json_data,
//    int json_data_len) {
//    DMReceiverRouter.eventUpdateFn(dm_instance_id, event_type,
//        json_data, json_data_len);
// }
// DLL_EXPORT void cmix_dm_set_router(DMReceiverRouterFunctions cbs) {
//     DMReceiverRouter.receiveFn = cbs.receiveFn;
//     DMReceiverRouter.receiveTextFn = cbs.receiveTextFn;
//     DMReceiverRouter.receiveReplyFn = cbs.receiveReplyFn;
//     DMReceiverRouter.receiveReactionFn = cbs.receiveReactionFn;
//     DMReceiverRouter.updateSentStatusFn = cbs.updateSentStatusFn;
//     DMReceiverRouter.blockSenderFn = cbs.blockSenderFn;
//     DMReceiverRouter.unblockSenderFn = cbs.unblockSenderFn;
//     DMReceiverRouter.getConversationFn = cbs.getConversationFn;
//     DMReceiverRouter.getConversationsFn = cbs.getConversationsFn;
//     DMReceiverRouter.deleteMessageFn = cbs.deleteMessageFn;
//     DMReceiverRouter.eventUpdateFn = cbs.eventUpdateFn;
// }
//
// /////////////////////////////////////////////////////////////////////////////
// //                                                                         //
// // Channels Go Callbacks                                                   //
// //                                                                         //
// // These are are called by go to call the functions defined in the         //
// // ChannelsRouter struct in the callbacks.h file. Do not modify.           //
// // They have to be defined                                                 //
// // here because of cgo rules.                                              //
// /////////////////////////////////////////////////////////////////////////////
//
// DLL_EXPORT int cmix_channels_get_id(int channels_instance_id) {
// 	 return ChannelsRouter.getIdFn(channels_instance_id);
// }
//
// DLL_EXPORT GoByteSlice cmix_channels_generate_channel_identity(int channels_instance_id, int cmix_id, GoError* err) {
// 	 return ChannelsRouter.generateChannelIdentityFn(channels_instance_id, cmix_id, err);
// }
//
// DLL_EXPORT GoByteSlice cmix_channels_construct_identity(int channels_instance_id, void* pubkey,int pubkey_len, int codeset, GoError* err) {
//	 return ChannelsRouter.constructIdentityFn(channels_instance_id, pubkey, pubkey_len, codeset,  err );
// }
//
// DLL_EXPORT GoByteSlice cmix_channels_import_private_identity(int channels_instance_id, char* password, int password_len, void* data, int data_len, GoError* err) {
//	 return ChannelsRouter.importPrivateIdentityFn(channels_instance_id, passsword, password_len, data, data_len, err);
// }
//
//
// DLL_EXPORT void cmix_channels_set_router(ChannelsRouterFunctions cbs) {
// 	ChannelsRouter.getIdFn = cbs.getIdFn;
//  ChannelsRouter.generateChannelIdentityFn = cbs.generateChannelIdentityFn;
//  ChannelsRouter.constructIdentityFn = cbs.constructIdentityFn
//	ChannelsRouter.importPrivateIdentityFn = cbs.importPrivateIdentityFn
// }

import "C"
