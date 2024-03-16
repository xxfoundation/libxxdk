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
// typedef struct {
//	int   IsError;
//	char* Msg;
//      int   MsgLen;
// } GoError;
//
// // below are the callbacks defined in callbacks.go
// extern long cmix_dm_receive(int dm_instance_id,
//    void* mesage_id, int message_id_len,
//    char* nickname, int nickname_len,
//    void* text, int text_len,
//    void* partnerkey, int partnerkey_len,
//    void* senderkey, int senderkey_len,
//    int dmToken, int codeset,
//    long timestamp, long round_id, long message_type, long status);
// extern long cmix_dm_receive_text(int dm_instance_id,
//    void* message_id, int message_id_len,
//    char* nickname, int nickname_len,
//    char* text, int text_len,
//    void* partnerkey, int partnerkey_len,
//    void* senderkey, int senderkey_len,
//    int dmToken, int codeset,
//    long timestamp, long round_id, long status);
// extern long cmix_dm_receive_reply(int dm_instance_id,
//    void* message_id, int message_id_len,
//    void* reply_to, int reply_to_len,
//    char* nickname, int nickname_len,
//    char* text, int text_len,
//    void* partnerkey, int partnerkey_len,
//    void* senderkey, int senderkey_len,
//    int dmToken, int codeset,
//    long timestamp, long round_id, long status);
// extern long cmix_dm_receive_reaction(int dm_instance_id,
//    void* message_id, int message_id_len,
//    void* reaction_to, int reaction_to_len,
//    char* nickname, int nickname_len,
//    char* text, int text_len,
//    void* partnerkey, int partnerkey_len,
//    void* senderkey, int senderkey_len,
//    int dmToken, int codeset,
//    long timestamp, long round_id, long status);
// extern void cmix_dm_update_sent_status(int dm_instance_id,
//    long uuid,
//    void* message_id, int message_id_len, long timestamp,
//    long round_id, long status);
// extern void cmix_dm_block_sender(int dm_instance_id,
//    void* pubkey, int pubkey_len);
// extern void cmix_dm_unblock_sender(int dm_instance_id,
//    void* pubkey, int pubkey_len);
// extern GoByteSlice cmix_dm_get_conversation(int dm_instance_id,
//    void* senderkey, int senderkey_len);
// extern GoByteSlice cmix_dm_get_conversations(int dm_instance_id);
// extern int cmix_dm_delete_message(void* message_id, int message_id_len,
//    void* pubkey, int pubkey_len);
// extern int cmix_dm_event_update(long event_type, void* json_data,
//    int json_data_len);
// extern void cmix_dm_set_router(DMReceiverRouterFunctions cbs);
import "C"

import (
	"fmt"
	"strings"

	"github.com/pkg/errors"
	jww "github.com/spf13/jwalterweatherman"
	"gitlab.com/elixxir/client/v4/bindings"
	"gitlab.com/elixxir/crypto/codename"
	"gitlab.com/elixxir/crypto/fastRNG"
	"gitlab.com/xx_network/crypto/csprng"
)

func makeError(e error) C.GoError {
	isErr := 0
	Msg := ""
	if e != nil {
		isErr = 1
		Msg = fmt.Sprintf("%+v", e)
	}
	return C.GoError{
		IsError: C.int(isErr),
		Msg:     C.CString(Msg),
		MsgLen:  C.int(len(Msg)),
	}
}

func makeBytes(s []byte) C.GoByteSlice {
	return C.GoByteSlice{
		len:  C.int(len(s)),
		data: C.CBytes(s),
	}
}

////////////////////////////////////////////////////////////////////////////////
//                                                                            //
// Core cMix Functionality                                                    //
//                                                                            //
////////////////////////////////////////////////////////////////////////////////

// GetVersion returns the xxdk.SEMVER.
//
//export GetVersion
func GetVersion() string {
	return bindings.GetVersion()
}

// GetGitVersion returns the xxdk.GITVERSION.
//
//export GetGitVersion
func GetGitVersion() string {
	return bindings.GetGitVersion()
}

// GetDependencies returns the xxdk.DEPENDENCIES.
//
//export GetDependencies
func GetDependencies() string {
	return bindings.GetDependencies()
}

// NewCmix creates user storage, generates keys, connects, and registers with
// the network. Note that this does not register a username/identity, but merely
// creates a new cryptographic identity for adding such information at a later
// date.
//
// Users of this function should delete the storage directory on error.
//
//export NewCmix
func NewCmix(ndfJSON, storageDir string, password []byte,
	registrationCode string) C.GoError {
	err := bindings.NewCmix(ndfJSON, storageDir, password, registrationCode)
	return makeError(err)
}

// LoadCmix will load an existing user storage from the storageDir using the
// password. This will fail if the user storage does not exist or the password
// is incorrect.
//
// The password is passed as a byte array so that it can be cleared from memory
// and stored as securely as possible using the MemGuard library.
//
// LoadCmix does not block on network connection and instead loads and starts
// subprocesses to perform network operations.
//
// This function returns a cMix Instance ID (int32) required to call
// specific cMix functions. If an error occurs, instance ID -1 is returned.
//
// Creating multiple cMix instance IDs with the same storage Dir will
// cause data corruption. In most cases only 1 instance should ever be
// needed.
//
//export LoadCmix
func LoadCmix(storageDir string, password []byte, cmixParamsJSON []byte) (int32,
	C.GoError) {
	// NOTE: we copy here because the elements need to persist.
	// We assume everything is freed after use when sent over the
	// c lib boundary.
	storageDirCpy := strings.Clone(storageDir)
	secret := make([]byte, len(password))
	copy(secret, password)
	cmixParams := make([]byte, len(cmixParamsJSON))
	copy(cmixParams, cmixParamsJSON)

	instance, err := bindings.LoadCmix(storageDirCpy, secret, cmixParams)
	if err != nil {
		return -1, makeError(err)
	}
	return int32(instance.GetID()), makeError(nil)
}

// cmix_GetReceptionID returns the current default reception ID
//
//export cmix_GetReceptionID
func cmix_GetReceptionID(cMixInstanceID int32) (C.GoByteSlice, C.GoError) {
	cMix, err := bindings.GetCMixInstance(int(cMixInstanceID))
	if err != nil {
		return makeBytes(nil), makeError(err)
	}
	return makeBytes(cMix.GetReceptionID()), makeError(nil)
}

//export cmix_EKVGet
func cmix_EKVGet(cMixInstanceID int32, key string) (C.GoByteSlice, C.GoError) {
	cMix, err := bindings.GetCMixInstance(int(cMixInstanceID))
	if err != nil {
		return makeBytes(nil), makeError(err)
	}
	val, err := cMix.EKVGet(key)
	return makeBytes(val), makeError(err)
}

//export cmix_EKVSet
func cmix_EKVSet(cMixInstanceID int32, key string, value []byte) C.GoError {
	cMix, err := bindings.GetCMixInstance(int(cMixInstanceID))
	if err != nil {
		return makeError(err)
	}
	return makeError(cMix.EKVSet(key, value))
}

//export cmix_StartNetworkFollower
func cmix_StartNetworkFollower(cMixInstanceID int32, timeoutMS int) C.GoError {
	cMix, err := bindings.GetCMixInstance(int(cMixInstanceID))
	if err != nil {
		return makeError(err)
	}
	return makeError(cMix.StartNetworkFollower(timeoutMS))
}

//export cmix_StopNetworkFollower
func cmix_StopNetworkFollower(cMixInstanceID int32) C.GoError {
	cMix, err := bindings.GetCMixInstance(int(cMixInstanceID))
	if err != nil {
		return makeError(err)
	}
	return makeError(cMix.StopNetworkFollower())
}

//export cmix_WaitForNetwork
func cmix_WaitForNetwork(cMixInstanceID int32, timeoutMS int) C.GoError {
	cMix, err := bindings.GetCMixInstance(int(cMixInstanceID))
	if err != nil {
		return makeError(err)
	}
	ok := cMix.WaitForNetwork(timeoutMS)
	if !ok {
		return makeError(errors.Errorf(
			"Timed out waiting for network"))
	}
	return makeError(nil)
}

//export cmix_ReadyToSend
func cmix_ReadyToSend(cMixInstanceID int32) bool {
	cmix, err := bindings.GetCMixInstance(int(cMixInstanceID))
	if err != nil {
		jww.ERROR.Printf("%+v", err)
		return false
	}
	return cmix.ReadyToSend()
}

////////////////////////////////////////////////////////////////////////////////
//                                                                            //
// Direct Messaging                                                           //
//                                                                            //
////////////////////////////////////////////////////////////////////////////////

//export cmix_GenerateCodenameIdentity
func cmix_GenerateCodenameIdentity(secretPassphrase string) C.GoByteSlice {
	// TODO: maybe a singleton or init func to this? is there a better
	// way to do this? would it ever make sense to take an RNG
	// from C?
	rngGen := fastRNG.NewStreamGenerator(1, 1, csprng.NewSystemRNG)
	rng := rngGen.GetStream()
	defer rng.Close()
	cn, err := codename.GenerateIdentity(rng)
	if err != nil {
		jww.FATAL.Panicf("%+v", err)
	}
	cnBytes, err := cn.Export(secretPassphrase, rng)
	if err != nil {
		jww.FATAL.Panicf("%+v", err)
	}
	jww.TRACE.Printf("Codename: %s", string(cnBytes))
	return makeBytes(cnBytes)
}

var dmReceivers map[int]*dmReceiver

//export cmix_dm_NewDMClient
func cmix_dm_NewDMClient(cMixInstanceID int32, codenameIdentity []byte,
	secretPassphrase string) (int32, C.GoError) {
	jww.TRACE.Printf("Received Codename: %s", string(codenameIdentity))
	pi, err := codename.ImportPrivateIdentity(secretPassphrase,
		codenameIdentity)
	if err != nil {
		return -1, makeError(err)
	}
	myReceiver := &dmReceiver{}
	receiver := bindings.NewDMReceiver(myReceiver)
	notifications, _ := bindings.LoadNotificationsDummy(int(cMixInstanceID))
	notificationsID := notifications.GetID()
	dmClient, err := bindings.NewDMClientWithGoEventModel(
		int(cMixInstanceID), notificationsID,
		pi.Marshal(), receiver, myReceiver)
	if err != nil {
		return -1, makeError(err)
	}

	// Set up receiver tracking
	if dmReceivers == nil {
		dmReceivers = make(map[int]*dmReceiver)
	}
	cid := dmClient.GetID()
	myReceiver.dmClientID = cid
	dmReceivers[cid] = myReceiver
	return int32(cid), makeError(nil)
}

//export cmix_dm_GetDMToken
func cmix_dm_GetDMToken(dmInstanceID int32) (int32, C.GoError) {
	dmClient, err := bindings.GetDMInstance(int(dmInstanceID))
	if err != nil {
		return 0, makeError(err)
	}
	return int32(dmClient.GetToken()), makeError(nil)
}

//export cmix_dm_GetDMPubKey
func cmix_dm_GetDMPubKey(dmInstanceID int32) (C.GoByteSlice, C.GoError) {
	dmClient, err := bindings.GetDMInstance(int(dmInstanceID))
	if err != nil {
		return makeBytes(nil), makeError(err)
	}
	return makeBytes(dmClient.GetPublicKey()), makeError(nil)
}

//export cmix_dm_Send
func cmix_dm_Send(dmInstanceID int32, partnerPubKey []byte,
	dmToken int32, messageType int, plaintext []byte, leaseTimeMS int64,
	cmixParamsJSON []byte) (C.GoByteSlice, C.GoError) {
	dmClient, err := bindings.GetDMInstance(int(dmInstanceID))
	if err != nil {
		return makeBytes(nil), makeError(err)
	}
	sendReportJSON, err := dmClient.Send(partnerPubKey, dmToken,
		messageType, plaintext, leaseTimeMS, cmixParamsJSON)
	return makeBytes(sendReportJSON), makeError(err)
}

//export cmix_dm_SendText
func cmix_dm_SendText(dmInstanceID int32, partnerPubKey []byte,
	dmToken int32, message string, leaseTimeMS int64,
	cmixParamsJSON []byte) (C.GoByteSlice, C.GoError) {
	dmClient, err := bindings.GetDMInstance(int(dmInstanceID))
	if err != nil {
		return makeBytes(nil), makeError(err)
	}
	sendReportJSON, err := dmClient.SendText(partnerPubKey, dmToken,
		message, leaseTimeMS, cmixParamsJSON)
	return makeBytes(sendReportJSON), makeError(err)
}

//export cmix_dm_SendReply
func cmix_dm_SendReply(dmInstanceID int32, partnerPubKey []byte,
	dmToken int32, message string, replyTo []byte, leaseTimeMS int64,
	cmixParamsJSON []byte) (C.GoByteSlice, C.GoError) {
	dmClient, err := bindings.GetDMInstance(int(dmInstanceID))
	if err != nil {
		return makeBytes(nil), makeError(err)
	}
	sendReportJSON, err := dmClient.SendReply(partnerPubKey, dmToken,
		message, replyTo, leaseTimeMS, cmixParamsJSON)
	return makeBytes(sendReportJSON), makeError(err)
}

//export cmix_dm_SendReaction
func cmix_dm_SendReaction(dmInstanceID int32, partnerPubKey []byte,
	dmToken int32, message string, reactTo []byte, leaseTimeMS int64,
	cmixParamsJSON []byte) (C.GoByteSlice, C.GoError) {
	dmClient, err := bindings.GetDMInstance(int(dmInstanceID))
	if err != nil {
		return makeBytes(nil), makeError(err)
	}
	sendReportJSON, err := dmClient.SendReaction(partnerPubKey, dmToken,
		message, reactTo, cmixParamsJSON)
	return makeBytes(sendReportJSON), makeError(err)
}

// This implements the bindings.DMReceiver interface.
type dmReceiver struct {
	dmClientID int
}

func (dmr *dmReceiver) Receive(messageID []byte, nickname string,
	text []byte, partnerKey, senderKey []byte, dmToken int32, codeset int,
	timestamp, roundId, mType, status int64) int64 {
	return int64(C.cmix_dm_receive(C.int(dmr.dmClientID),
		C.CBytes(messageID), C.int(len(messageID)),
		C.CString(nickname), C.int(len(nickname)),
		C.CBytes(text), C.int(len(text)),
		C.CBytes(partnerKey), C.int(len(partnerKey)),
		C.CBytes(senderKey), C.int(len(senderKey)),
		C.int(dmToken),
		C.int(codeset), C.long(timestamp), C.long(roundId),
		C.long(mType), C.long(status)))
}

func (dmr *dmReceiver) ReceiveText(messageID []byte,
	nickname, text string, partnerKey, senderKey []byte, dmToken int32, codeset int,
	timestamp, roundId, status int64) int64 {
	return int64(C.cmix_dm_receive_text(C.int(dmr.dmClientID),
		C.CBytes(messageID), C.int(len(messageID)),
		C.CString(nickname), C.int(len(nickname)),
		C.CString(text), C.int(len(text)),
		C.CBytes(partnerKey), C.int(len(partnerKey)),
		C.CBytes(senderKey), C.int(len(senderKey)),
		C.int(dmToken),
		C.int(codeset), C.long(timestamp), C.long(roundId),
		C.long(status)))
}

func (dmr *dmReceiver) ReceiveReply(messageID, replyTo []byte,
	nickname, text string, partnerKey, senderKey []byte, dmToken int32,
	codeset int, timestamp, roundId, status int64) int64 {
	return int64(C.cmix_dm_receive_reply(C.int(dmr.dmClientID),
		C.CBytes(messageID), C.int(len(messageID)),
		C.CBytes(replyTo), C.int(len(replyTo)),
		C.CString(nickname), C.int(len(nickname)),
		C.CString(text), C.int(len(text)),
		C.CBytes(partnerKey), C.int(len(partnerKey)),
		C.CBytes(senderKey), C.int(len(senderKey)),
		C.int(dmToken),
		C.int(codeset), C.long(timestamp), C.long(roundId),
		C.long(status)))
}

func (dmr *dmReceiver) ReceiveReaction(messageID, reactionTo []byte,
	nickname, reaction string, partnerKey, senderKey []byte, dmToken int32,
	codeset int, timestamp, roundId, status int64) int64 {
	return int64(C.cmix_dm_receive_reaction(C.int(dmr.dmClientID),
		C.CBytes(messageID), C.int(len(messageID)),
		C.CBytes(reactionTo), C.int(len(reactionTo)),
		C.CString(nickname), C.int(len(nickname)),
		C.CString(reaction), C.int(len(reaction)),
		C.CBytes(partnerKey), C.int(len(partnerKey)),
		C.CBytes(senderKey), C.int(len(senderKey)),
		C.int(dmToken),
		C.int(codeset), C.long(timestamp), C.long(roundId),
		C.long(status)))
}

func (dmr *dmReceiver) UpdateSentStatus(uuid int64, messageID []byte,
	timestamp, roundID, status int64) {
	C.cmix_dm_update_sent_status(C.int(dmr.dmClientID),
		C.long(uuid), C.CBytes(messageID),
		C.int(len(messageID)),
		C.long(timestamp), C.long(roundID), C.long(status))
}

func (dmr *dmReceiver) BlockSender(pubKey []byte) {
	C.cmix_dm_block_sender(C.int(dmr.dmClientID), C.CBytes(pubKey),
		C.int(len(pubKey)))
}

func (dmr *dmReceiver) UnblockSender(pubKey []byte) {
	C.cmix_dm_unblock_sender(C.int(dmr.dmClientID), C.CBytes(pubKey),
		C.int(len(pubKey)))
}

func (dmr *dmReceiver) GetConversation(senderPubKey []byte) []byte {
	data2Copy := C.cmix_dm_get_conversation(C.int(dmr.dmClientID),
		C.CBytes(senderPubKey), C.int(len(senderPubKey)))
	res := make([]byte, data2Copy.len)
	buf := *(*[]byte)(data2Copy.data)
	len := data2Copy.len
	copy(res, buf[0:len])
	return res
}

func (dmr *dmReceiver) GetConversations() []byte {
	data2Copy := C.cmix_dm_get_conversations(C.int(dmr.dmClientID))
	res := make([]byte, data2Copy.len)
	buf := *(*[]byte)(data2Copy.data)
	len := data2Copy.len
	copy(res, buf[0:len])
	return res
}

func (dmr *dmReceiver) DeleteMessage(messageID, senderPubKey []byte) bool {
	res := int(C.cmix_dm_delete_message(
		C.CBytes(messageID), C.int(len(messageID)),
		C.CBytes(senderPubKey), C.int(len(senderPubKey))))
	if res == 0 {
		return false
	} else {
		return true
	}
}

func (dmr *dmReceiver) EventUpdate(eventType int64, jsonData []byte) {
	C.cmix_dm_event_update(C.long(eventType),
		C.CBytes(jsonData), C.int(len(jsonData)))
}
func main() {}
