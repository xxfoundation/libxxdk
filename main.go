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
import "C"

import (
	"unsafe"

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
	registrationCode string) error {
	return bindings.NewCmix(ndfJSON, storageDir, password, registrationCode)
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
	error) {
	instance, err := bindings.LoadCmix(storageDir, password, cmixParamsJSON)
	if err != nil {
		return -1, err
	}
	return int32(instance.GetID()), nil
}

// cmix_GetReceptionID returns the current default reception ID
//
//export cmix_GetReceptionID
func cmix_GetReceptionID(cMixInstanceID int32) ([]byte, error) {
	cMix, ok := bindings.GetCMixInstance(int(cMixInstanceID))
	if !ok {
		return nil, errors.Errorf(ErrInvalidCMixInstanceID,
			cMixInstanceID)
	}
	return cMix.GetReceptionID(), nil
}

//export cmix_EKVGet
func cmix_EKVGet(cMixInstanceID int32, key string) ([]byte, error) {
	cMix, ok := bindings.GetCMixInstance(int(cMixInstanceID))
	if !ok {
		return nil, errors.Errorf(ErrInvalidCMixInstanceID,
			cMixInstanceID)
	}
	return cMix.EKVGet(key)
}

//export cmix_EKVSet
func cmix_EKVSet(cMixInstanceID int32, key string, value []byte) error {
	cMix, ok := bindings.GetCMixInstance(int(cMixInstanceID))
	if !ok {
		return errors.Errorf(ErrInvalidCMixInstanceID,
			cMixInstanceID)
	}
	return cMix.EKVSet(key, value)
}

//export cmix_StartNetworkFollower
func cmix_StartNetworkFollower(cMixInstanceID, timeoutMS int) error {
	cmix, ok := bindings.GetCMixInstance(cMixInstanceID)
	if ok {
		return errors.Errorf(ErrInvalidCMixInstanceID, cMixInstanceID)
	}
	return cmix.StartNetworkFollower(timeoutMS)
}

//export cmix_StopNetworkFollower
func cmix_StopNetworkFollower(cMixInstanceID int) error {
	cmix, ok := bindings.GetCMixInstance(cMixInstanceID)
	if ok {
		return errors.Errorf(ErrInvalidCMixInstanceID, cMixInstanceID)
	}
	return cmix.StopNetworkFollower()
}

//export cmix_WaitForNetwork
func cmix_WaitForNetwork(cMixInstanceID, timeoutMS int) error {
	cmix, ok := bindings.GetCMixInstance(cMixInstanceID)
	if ok {
		return errors.Errorf(ErrInvalidCMixInstanceID, cMixInstanceID)
	}
	ok = cmix.WaitForNetwork(timeoutMS)
	if !ok {
		return errors.Errorf("Timed out waiting for network")
	}
	return nil
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
	secretPassphrase string) (int, error) {
	pi, err := codename.ImportPrivateIdentity(secretPassphrase,
		codenameIdentity)
	if err != nil {
		return -1, err
	}
	myReceiver := &dmReceiver{}
	receiver := bindings.NewDMReceiver(myReceiver)
	dmClient, err := bindings.NewDMClientWithGoEventModel(cMixInstanceID,
		pi.Marshal(), receiver)
	if err != nil {
		return -1, err
	}

	// Set up receiver tracking
	if dmReceivers == nil {
		dmReceivers = make(map[int]*dmReceiver)
	}
	cid := dmClient.GetID()
	myReceiver.dmClientID = cid
	dmReceivers[cid] = myReceiver
	return cid, nil
}

//export cmix_dm_GetDMToken
func cmix_dm_GetDMToken(dmInstanceID int) (uint32, error) {
	dmClient, ok := bindings.GetDMInstance(dmInstanceID)
	if !ok {
		return 0, errors.Errorf(ErrInvalidDMInstanceID,
			dmInstanceID)
	}
	return dmClient.GetToken(), nil
}

//export cmix_dm_GetDMPubKey
func cmix_dm_GetDMPubKey(dmInstanceID int) ([]byte, error) {
	dmClient, ok := bindings.GetDMInstance(dmInstanceID)
	if !ok {
		return nil, errors.Errorf(ErrInvalidDMInstanceID,
			dmInstanceID)
	}
	return dmClient.GetPublicKey(), nil
}

//export cmix_dm_SendText
func cmix_dm_SendText(dmInstanceID int, partnerPubKey []byte,
	dmToken uint32, message string, leaseTimeMS int64,
	cmixParamsJSON []byte) ([]byte, error) {
	dmClient, ok := bindings.GetDMInstance(dmInstanceID)
	if !ok {
		return nil, errors.Errorf(ErrInvalidDMInstanceID,
			dmInstanceID)
	}
	return dmClient.SendText(partnerPubKey, dmToken,
		message, leaseTimeMS, cmixParamsJSON)
}

type ReceiveCallback func(dmClientID C.int, messageID unsafe.Pointer,
	messageIDLen C.int, nickname *C.char, nicknameLen C.int,
	text unsafe.Pointer, textLen C.int,
	pubKey unsafe.Pointer, pubKeyLen C.int,
	dmToken C.int, codeset C.int, timestamp C.long, roundId C.long,
	mType C.long, status C.long) C.long

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
