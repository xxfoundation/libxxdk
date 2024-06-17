////////////////////////////////////////////////////////////////////////////////
// Copyright © 2022 xx foundation                                             //
//                                                                            //
// Use of this source code is governed by a license that can be found in the  //
// LICENSE file.                                                              //
////////////////////////////////////////////////////////////////////////////////

package main

import (
	"sync"

	"gitlab.com/elixxir/client/v4/bindings"
)

//const rpcResponseChSzw = 5

type rpcCbs struct {
	response func(response []byte)
	errorFn  func(errorStr string)
}

func (r *rpcCbs) Response(response []byte) { r.response(response) }
func (r *rpcCbs) Error(errStr string)      { r.errorFn(errStr) }

type rpcServerCb struct {
	cb func(sender, request []byte) []byte
}

func (r *rpcServerCb) Callback(sender, request []byte) []byte {
	return r.cb(sender, request)
}

var rpcLock sync.Mutex
var rpcResponses = make(map[int32]bindings.RPCResponse)
var curRPCResponseID = int32(0)
var rpcServers = make(map[int32]bindings.RPCServer)
var curRPCServerID = int32(0)
