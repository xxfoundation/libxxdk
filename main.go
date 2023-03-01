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
//    void* pubkey, int pubkey_len,
//    int dmToken, int codeset,
//    long timestamp, long round_id, long message_type, long status);
// extern long cmix_dm_receive_text(int dm_instance_id,
//    void* message_id, int message_id_len,
//    char* nickname, int nickname_len,
//    char* text, int text_len,
//    void* pubkey, int pubkey_len,
//    int dmToken, int codeset,
//    long timestamp, long round_id, long status);
// extern long cmix_dm_receive_reply(int dm_instance_id,
//    void* message_id, int message_id_len,
//    void* reply_to, int reply_to_len,
//    char* nickname, int nickname_len,
//    char* text, int text_len,
//    void* pubkey, int pubkey_len,
//    int dmToken, int codeset,
//    long timestamp, long round_id, long status);
// extern long cmix_dm_receive_reaction(int dm_instance_id,
//    void* message_id, int message_id_len,
//    void* reaction_to, int reaction_to_len,
//    char* nickname, int nickname_len,
//    char* text, int text_len,
//    void* pubkey, int pubkey_len,
//    int dmToken, int codeset,
//    long timestamp, long round_id, long status);
// extern void cmix_dm_update_sent_status(int dm_instance_id,
//    long uuid,
//    void* message_id, int message_id_len, long timestamp,
//    long round_id, long status);
// extern void cmix_dm_set_callbacks(DMReceiverCallbackFunctions cbs);
import "C"

import (
	"fmt"

	"github.com/pkg/errors"
	jww "github.com/spf13/jwalterweatherman"
	"gitlab.com/elixxir/client/v4/bindings"
	"gitlab.com/elixxir/crypto/codename"
	"gitlab.com/elixxir/crypto/fastRNG"
	"gitlab.com/xx_network/crypto/csprng"
)

const (
	ErrInvalidCMixInstanceID = "Invalid cMix Instance ID: %d"
	ErrInvalidDMInstanceID   = "Invalid DM Instance ID: %d"
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
	instance, err := bindings.LoadCmix(storageDir, password, cmixParamsJSON)
	if err != nil {
		return -1, makeError(err)
	}
	return int32(instance.GetID()), makeError(nil)
}

// cmix_GetReceptionID returns the current default reception ID
//
//export cmix_GetReceptionID
func cmix_GetReceptionID(cMixInstanceID int32) ([]byte, C.GoError) {
	cMix, ok := bindings.GetCMixInstance(int(cMixInstanceID))
	if !ok {
		return nil, makeError(errors.Errorf(ErrInvalidCMixInstanceID,
			cMixInstanceID))
	}
	return cMix.GetReceptionID(), makeError(nil)
}

//export cmix_EKVGet
func cmix_EKVGet(cMixInstanceID int32, key string) ([]byte, C.GoError) {
	cMix, ok := bindings.GetCMixInstance(int(cMixInstanceID))
	if !ok {
		return nil, makeError(errors.Errorf(ErrInvalidCMixInstanceID,
			cMixInstanceID))
	}
	val, err := cMix.EKVGet(key)
	return val, makeError(err)
}

//export cmix_EKVSet
func cmix_EKVSet(cMixInstanceID int32, key string, value []byte) C.GoError {
	cMix, ok := bindings.GetCMixInstance(int(cMixInstanceID))
	if !ok {
		return makeError(errors.Errorf(ErrInvalidCMixInstanceID,
			cMixInstanceID))
	}
	return makeError(cMix.EKVSet(key, value))
}

//export cmix_StartNetworkFollower
func cmix_StartNetworkFollower(cMixInstanceID, timeoutMS int) C.GoError {
	cmix, ok := bindings.GetCMixInstance(cMixInstanceID)
	if ok {
		return makeError(errors.Errorf(ErrInvalidCMixInstanceID,
			cMixInstanceID))
	}
	return makeError(cmix.StartNetworkFollower(timeoutMS))
}

//export cmix_StopNetworkFollower
func cmix_StopNetworkFollower(cMixInstanceID int) C.GoError {
	cmix, ok := bindings.GetCMixInstance(cMixInstanceID)
	if ok {
		return makeError(errors.Errorf(ErrInvalidCMixInstanceID,
			cMixInstanceID))
	}
	return makeError(cmix.StopNetworkFollower())
}

//export cmix_WaitForNetwork
func cmix_WaitForNetwork(cMixInstanceID, timeoutMS int) C.GoError {
	cmix, ok := bindings.GetCMixInstance(cMixInstanceID)
	if ok {
		return makeError(errors.Errorf(ErrInvalidCMixInstanceID,
			cMixInstanceID))
	}
	ok = cmix.WaitForNetwork(timeoutMS)
	if !ok {
		return makeError(errors.Errorf(
			"Timed out waiting for network"))
	}
	return makeError(nil)
}

//export cmix_ReadyToSend
func cmix_ReadyToSend(cMixInstanceID int) bool {
	cmix, ok := bindings.GetCMixInstance(cMixInstanceID)
	if ok {
		jww.ERROR.Printf(ErrInvalidCMixInstanceID, cMixInstanceID)
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
func cmix_GenerateCodenameIdentity(secretPassphrase string) []byte {
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
	return cnBytes
}

var dmReceivers map[int]*dmReceiver

//export cmix_dm_NewDMClient
func cmix_dm_NewDMClient(cMixInstanceID int, codenameIdentity []byte,
	secretPassphrase string) (int, C.GoError) {
	pi, err := codename.ImportPrivateIdentity(secretPassphrase,
		codenameIdentity)
	if err != nil {
		return -1, makeError(err)
	}
	myReceiver := &dmReceiver{}
	receiver := bindings.NewDMReceiver(myReceiver)
	dmClient, err := bindings.NewDMClientWithGoEventModel(cMixInstanceID,
		pi.Marshal(), receiver)
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
	return cid, makeError(nil)
}

//export cmix_dm_GetDMToken
func cmix_dm_GetDMToken(dmInstanceID int) (uint32, C.GoError) {
	dmClient, ok := bindings.GetDMInstance(dmInstanceID)
	if !ok {
		return 0, makeError(errors.Errorf(ErrInvalidDMInstanceID,
			dmInstanceID))
	}
	return dmClient.GetToken(), makeError(nil)
}

//export cmix_dm_GetDMPubKey
func cmix_dm_GetDMPubKey(dmInstanceID int) ([]byte, C.GoError) {
	dmClient, ok := bindings.GetDMInstance(dmInstanceID)
	if !ok {
		return nil, makeError(errors.Errorf(ErrInvalidDMInstanceID,
			dmInstanceID))
	}
	return dmClient.GetPublicKey(), makeError(nil)
}

//export cmix_dm_SendText
func cmix_dm_SendText(dmInstanceID int, partnerPubKey []byte,
	dmToken uint32, message string, leaseTimeMS int64,
	cmixParamsJSON []byte) ([]byte, C.GoError) {
	dmClient, ok := bindings.GetDMInstance(dmInstanceID)
	if !ok {
		return nil, makeError(errors.Errorf(ErrInvalidDMInstanceID,
			dmInstanceID))
	}
	msgID, err := dmClient.SendText(partnerPubKey, dmToken,
		message, leaseTimeMS, cmixParamsJSON)
	return msgID, makeError(err)
}

// This implements the bindings.DMReceiver interface.
type dmReceiver struct {
	dmClientID int
}

func (dmr *dmReceiver) Receive(messageID []byte, nickname string,
	text []byte, pubKey []byte, dmToken int32, codeset int,
	timestamp, roundId, mType, status int64) int64 {
	return int64(C.cmix_dm_receive(C.int(dmr.dmClientID),
		C.CBytes(messageID), C.int(len(messageID)),
		C.CString(nickname), C.int(len(nickname)),
		C.CBytes(text), C.int(len(text)),
		C.CBytes(pubKey), C.int(len(pubKey)),
		C.int(dmToken),
		C.int(codeset), C.long(timestamp), C.long(roundId),
		C.long(mType), C.long(status)))
}

func (dmr *dmReceiver) ReceiveText(messageID []byte,
	nickname, text string, pubKey []byte, dmToken int32, codeset int,
	timestamp, roundId, status int64) int64 {
	return int64(C.cmix_dm_receive_text(C.int(dmr.dmClientID),
		C.CBytes(messageID), C.int(len(messageID)),
		C.CString(nickname), C.int(len(nickname)),
		C.CString(text), C.int(len(text)),
		C.CBytes(pubKey), C.int(len(pubKey)),
		C.int(dmToken),
		C.int(codeset), C.long(timestamp), C.long(roundId),
		C.long(status)))
}

func (dmr *dmReceiver) ReceiveReply(messageID, replyTo []byte,
	nickname, text string, pubKey []byte, dmToken int32,
	codeset int, timestamp, roundId, status int64) int64 {
	return int64(C.cmix_dm_receive_reply(C.int(dmr.dmClientID),
		C.CBytes(messageID), C.int(len(messageID)),
		C.CBytes(replyTo), C.int(len(replyTo)),
		C.CString(nickname), C.int(len(nickname)),
		C.CString(text), C.int(len(text)),
		C.CBytes(pubKey), C.int(len(pubKey)),
		C.int(dmToken),
		C.int(codeset), C.long(timestamp), C.long(roundId),
		C.long(status)))
}

func (dmr *dmReceiver) ReceiveReaction(messageID, reactionTo []byte,
	nickname, reaction string, pubKey []byte, dmToken int32,
	codeset int, timestamp, roundId, status int64) int64 {
	return int64(C.cmix_dm_receive_reaction(C.int(dmr.dmClientID),
		C.CBytes(messageID), C.int(len(messageID)),
		C.CBytes(reactionTo), C.int(len(reactionTo)),
		C.CString(nickname), C.int(len(nickname)),
		C.CString(reaction), C.int(len(reaction)),
		C.CBytes(pubKey), C.int(len(pubKey)),
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

func main() {}
