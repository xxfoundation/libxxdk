using System;
using System.Buffers.Text;
using System.Diagnostics.Metrics;
using System.Net.Sockets;
using System.Runtime.InteropServices;
using System.Runtime.Intrinsics.X86;
using System.Security.Principal;
using System.Text;
using System.CommandLine;
using System.Reflection.PortableExecutable;
using System.Reflection;
using System.Net.NetworkInformation;

namespace XX;

// Mappable system types
using GoInt8 = System.SByte;
using GoUint8 = System.Byte;
using GoInt16 = System.Int16;
using GoUint16 = System.UInt16;
using GoInt32 = System.Int32;
using GoUint32 = System.UInt32;
using GoInt64 = System.Int64;
using GoUint64 = System.UInt64;
using GoInt = System.Int64;
using GoUint = System.UInt64;
using GoUintptr = System.UIntPtr;
using GoFloat32 = System.Single;
using GoFloat64 = System.Double;

// GoString represents a String in
// golang and allows us to pass strings to
// the external shared library function as
// parameter values.
struct GoString
{
    public IntPtr p;
    public Int64 n;
}

// GoSlice represents
// a slice in golang so that we
// can pass slices to the go external
// shared library.
struct GoSlice
{
    public IntPtr data;
    public Int64 len;
    public Int64 cap;
}

// GoInterface represents an error object in golang. 
[StructLayout(LayoutKind.Sequential)]
struct GoError
{
    public Int32 IsError;
    public IntPtr Msg;
    public Int32 MsgLen;
}
/// <summary>
/// GoByteSlice is a byte slice in the go format. Do not use unless
/// internally to this module to implement a callback return to go. 
/// </summary>
[StructLayout(LayoutKind.Sequential)]
public struct GoByteSlice
{
    /// <summary>
    /// Length of the slice
    /// </summary>
    public Int64 len;
    /// <summary>
    /// Slice data
    /// </summary>
    public IntPtr data;
}
/* Return type for LoadCmix */
[StructLayout(LayoutKind.Sequential)]
struct LoadCmix_return
{
    public GoInt32 cMixInstanceID;
    public GoError Err;
}
/* Return type for cmix_GetReceptionID */
[StructLayout(LayoutKind.Sequential)]
struct cmix_GetReceptionID_return
{
    public GoByteSlice receptionID;
    public GoError Err;
}
/* Return type for cmix_EKVGet */
[StructLayout(LayoutKind.Sequential)]
struct cmix_EKVGet_return
{
    public GoByteSlice Val;
    public GoError Err;
}
/* Return type for cmix_dm_NewDMClient */
[StructLayout(LayoutKind.Sequential)]
struct cmix_dm_NewDMClient_return
{
    public GoInt32 DMInstanceID;
    public GoError Err;
}
/* Return type for cmix_dm_GetDMToken */
[StructLayout(LayoutKind.Sequential)]
struct cmix_dm_GetDMToken_return
{
    public GoInt32 Token;
    public GoError Err;
}
/* Return type for cmix_dm_GetDMPubKey */
[StructLayout(LayoutKind.Sequential)]
struct cmix_dm_GetDMPubKey_return
{
    public GoByteSlice PubKey;
    public GoError Err;
}
/* Return type for cmix_dm_SendText */
[StructLayout(LayoutKind.Sequential)]
struct cmix_dm_SendText_return
{
    public GoByteSlice SendReportJSON;
    public GoError Err;
}

/// <summary>
/// XX Network functions and classes
/// </summary>
public unsafe class Network
{
    /// <summary>
    /// Version of XX Network Library
    /// </summary>
    /// <returns>version string</returns>
    public static String GetVersion()
    {
        return ConvertGoString(CLIB.GetVersion());
    }

    /// <summary>
    /// GetGitVersion returns the xxdk.GITVERSION.
    /// </summary>
    /// <returns>git commit version string</returns>
    public static String GetGitVersion()
    {
        return ConvertGoString(CLIB.GetGitVersion());
    }

    /// <summary>
    /// GetDependencies returns the xxdk.DEPENDENCIES.
    /// </summary>
    /// <returns>Returns the go dependencies list as a string</returns>
    public static String GetDependencies()
    {
        return ConvertGoString(CLIB.GetDependencies());
    }

    /// <summary>
    /// Generates a "Codename" Identity. These are used for Direct Messages
    /// and channels.
    /// </summary>
    /// <param name="secretPassphrase">The encryption password for the
    /// identity blob</param>
    /// <returns>An encoded and encrypted ID object, printable as a
    /// string</returns>
    public static Byte[] GenerateCodenameIdentity(
        String secretPassphrase)
    {
        GoString secret = NewGoString(secretPassphrase);
        GoByteSlice id = CLIB.cmix_GenerateCodenameIdentity(secret);
        FreeGoString(secret);
        return GoByteSliceToBytes(id);
    }

    /// <summary>
    /// cMix Functions for connecting with the XX network
    /// </summary>
    public class CMix
    {
        private Int32 cMixInstanceID;

        // Use LoadCmix to get a proper CMix object.
        // NewCmix must first be called to instantiate the object.
        private CMix()
        {
            this.cMixInstanceID = -1;
        }

        /// <summary>Return the instance id handler used by the cMix library.
        /// -1 indicates an uninalizated insance.</summary>
        public Int32 GetInstanceID()
        {
            return this.cMixInstanceID;
        }

        /// <summary>
        /// Create a new cMix instantiation. This does not
        /// connect to the network, but merely sets up storage and
        /// validates the network definition file. 
        /// </summary>
        /// <param name="ndfJSON">network definition file</param>
        /// <param name="storageDir">directory to store state</param>
        /// <param name="password">encryption password for the user</param>
        /// <param name="registrationCode">Optional, used to register with
        /// restricted networks</param>
        /// <exception cref="Exception">error is returned by the library</exception>
        public static void NewCmix(String ndfJSON,
            String storageDir, Byte[] password,
            string registrationCode)
        {
            GoString ndfJSONGS = NewGoString(ndfJSON);
            GoString storageDirGS = NewGoString(storageDir);
            GoSlice secret = NewGoSlice(password);
            GoString regCode = NewGoString(registrationCode);

            GoError err = CLIB.NewCmix(ndfJSONGS, storageDirGS, secret,
                regCode);

            FreeGoSlice(secret);
            FreeGoString(regCode);
            FreeGoString(ndfJSONGS);
            FreeGoString(storageDirGS);

            if (err.IsError != 0)
            {
                String errMsg = ConvertCharPtr(err.Msg, err.MsgLen);
                throw new Exception(errMsg);
            }

        }

        /// <summary>
        /// Connect to the XX Network and create a cMix Instance
        /// </summary>
        /// <param name="storageDir">state folder</param>
        /// <param name="password">encryption password for state folder</param>
        /// <param name="cmixParamsJSON">Optional. Parameters to use, defaults
        /// should be used unless you are testing.</param>
        /// <returns>CMix Instance Identifier (a number, acts
        /// like a file descriptor)</returns>
        /// <exception cref="Exception">Errors occured during setup</exception>
        public static CMix LoadCmix(String storageDir, Byte[] password,
            Byte[] cmixParamsJSON)
        {
            GoString storageDirGS = NewGoString(storageDir);
            GoSlice secret = NewGoSlice(password);
            GoSlice cmixParams = NewGoSlice(cmixParamsJSON);

            LoadCmix_return ret = CLIB.LoadCmix(storageDirGS, secret,
                cmixParams);

            FreeGoSlice(cmixParams);
            FreeGoSlice(secret);
            FreeGoString(storageDirGS);

            GoError err = ret.Err;
            if (err.IsError != 0)
            {
                String errMsg = ConvertCharPtr(err.Msg, err.MsgLen);
                throw new Exception(errMsg);
            }

            CMix cMix = new CMix();
            cMix.cMixInstanceID = ret.cMixInstanceID;
            return cMix;
        }

        /// <summary>
        /// cmix_GetReceptionID returns the current default reception ID
        /// </summary>
        /// <returns>The cMix Reception ID, used to receive RAW cMix Messages</returns>
        /// <exception cref="Exception">Error occured in library</exception>
        public Byte[] GetReceptionID()
        {
            cmix_GetReceptionID_return rid = CLIB.cmix_GetReceptionID(
                this.cMixInstanceID);
            GoError err = rid.Err;
            if (err.IsError != 0)
            {
                String errMsg = ConvertCharPtr(err.Msg, err.MsgLen);
                throw new Exception(errMsg);
            }
            return GoByteSliceToBytes(rid.receptionID);
        }

        /// <summary>
        /// EKVGet retrieves a value from secure Key-Value storage.
        /// </summary>
        /// <param name="key">The key associated with the value</param>
        /// <returns>Bytes of the value</returns>
        /// <exception cref="Exception">Error or if key cannot be found</exception>
        public Byte[] EKVGet(String key)
        {
            GoString goKey = NewGoString(key);
            cmix_EKVGet_return ret = CLIB.cmix_EKVGet(this.cMixInstanceID, goKey);

            GoError err = ret.Err;
            if (err.IsError != 0)
            {
                String errMsg = ConvertCharPtr(err.Msg, err.MsgLen);
                throw new Exception(errMsg);
            }
            return GoByteSliceToBytes(ret.Val);
        }

        /// <summary>
        /// EKVSet a value in the Key value storage
        /// </summary>
        /// <param name="key">The key associated with the value</param>
        /// <param name="value">The bytes to store as the value</param>
        /// <exception cref="Exception">Write errors</exception>
        public void EKVSet(String key,
            Byte[] value)
        {
            GoString goKey = NewGoString(key);
            GoSlice goVal = NewGoSlice(value);
            GoError err = CLIB.cmix_EKVSet(this.cMixInstanceID, goKey, goVal);
            if (err.IsError != 0)
            {
                String errMsg = ConvertCharPtr(err.Msg, err.MsgLen);
                throw new Exception(errMsg);
            }

            FreeGoSlice(goVal);
            FreeGoString(goKey);
        }

        /// <summary>
        /// Start following the cMix network. 
        /// </summary>
        /// <param name="timeoutMS">Error out if we haven't connected
        /// by this many milliseconds</param>
        /// <exception cref="Exception">Timeout or other error on
        /// connection</exception>
        public void StartNetworkFollower(Int32 timeoutMS)
        {
            GoError err = CLIB.cmix_StartNetworkFollower(this.cMixInstanceID,
                timeoutMS);
            if (err.IsError != 0)
            {
                String errMsg = ConvertCharPtr(err.Msg, err.MsgLen);
                throw new Exception(errMsg);
            }
        }
        /// <summary>
        /// Stop the network follower. You should call this any time
        /// your app goes to the background. 
        /// </summary>
        /// <exception cref="Exception">Timeout or other error on
        /// shutdown</exception>
        public void StopNetworkFollower()
        {
            GoError err = CLIB.cmix_StopNetworkFollower(cMixInstanceID);
            if (err.IsError != 0)
            {
                String errMsg = ConvertCharPtr(err.Msg, err.MsgLen);
                throw new Exception(errMsg);
            }
        }
        /// <summary>
        /// Wait timeoutMS or until you are caught up with the network.
        /// Use this before you send a message.
        /// </summary>
        /// <param name="timeoutMS">Timeout in milliseconds</param>
        /// <exception cref="Exception">Timed out or other send
        /// error</exception>
        public void WaitForNetwork(Int32 timeoutMS)
        {
            GoError err = CLIB.cmix_WaitForNetwork(this.cMixInstanceID, timeoutMS);
            if (err.IsError != 0)
            {
                String errMsg = ConvertCharPtr(err.Msg, err.MsgLen);
                throw new Exception(errMsg);
            }
        }
        /// <summary>
        /// Are we ready to send? This is a quick check you can make. Useful
        /// for showing the UI if you are connected.
        /// </summary>
        /// <returns>If we are connected</returns>
        public Boolean ReadyToSend()
        {
            GoUint8 ready = CLIB.cmix_ReadyToSend(this.cMixInstanceID);
            if (ready != 0)
            {
                return true;
            }
            return false;
        }
    }

    /// <summary>
    /// IDMCallbackFunctions is the interface for DM Callbacks. Users
    /// must pass an impelementation of this interface to the DM object.
    /// </summary>
    public interface IDMReceiver
    {
        /// <summary>
        /// Receive RAW direct message callback
        /// </summary>
        Int64 Receive(Byte[] message_id, String nickname,
            Byte[] text, Byte[] partnerkey, Byte[] senderkey, Int32 dmToken,
            Int32 codeset, Int64 timestamp, Int64 round_id, Int64 msg_type,
            Int64 status);

        /// <summary>
        /// Received Text message callback
        /// </summary>
        Int64 ReceiveText(Byte[] message_id, String nickname,
            String text, Byte[] partnerkey, Byte[] senderkey, Int32 dmToken,
            Int32 codeset, Int64 timestamp, Int64 round_id, Int64 status);

        /// <summary>
        /// Received Reply message callback
        /// </summary>
        Int64 ReceiveReply(Byte[] message_id, Byte[] reply_to,
            String nickname, String text, Byte[] partnerkey, Byte[] senderkey,
            Int32 dmToken, Int32 codeset, Int64 timestamp, Int64 round_id,
            Int64 status);

        /// <summary>
        /// Received Reaction message callback
        /// </summary>
        Int64 ReceiveReaction(Byte[] message_id, Byte[] reaction_to,
            String nickname, String text, Byte[] partnerkey, Byte[] senderkey,
            Int32 dmToken, Int32 codeset, Int64 timestamp, Int64 round_id,
            Int64 status);

        /// <summary>
        /// Message was updated callback. Used to tell UI progress as
        /// message is sent through the network. 
        /// </summary>
        Int64 UpdateSentStatus(Int64 uuid, Byte[] message_id,
            Int64 timestamp, Int64 round_id, Int64 status);

        /// <summary>
        /// User is blocked callback. Used to tell UI a user is blocked.
        /// </summary>
        void BlockUser(Byte[] pubkey);

        /// <summary>
        /// User is unblocked callback. Used to tell UI a user is unblocked.
        /// </summary>
        void UnblockUser(Byte[] pubkey);

        /// <summary>
        /// GetConversation callback. Used to retrieve conversation object.
        /// </summary>
        Byte[] GetConversation(Byte[] senderkey);

        /// <summary>
        /// GetConversations callback. Used to retrieve all conversation objects.
        /// </summary>
        Byte[] GetConversations();

        /// <summary>
        /// DeleteMessage callback. Used to signal a delete request.
        /// </summary>
        bool DeleteMessage(Byte[] MessageID, Byte[] pubkey);

        /// <summary>
        /// EventUpdate callback. Used to signal an event
        /// </summary>
        void EventUpdate(Int64 eventType, Byte[] jsonData);
    }

    /// <summary>
    /// DMReceiverRounder is used by the c library callback functions
    /// (implemented below) to route to and call the registered DMReceiver
    /// functions for a given DM Instance.
    /// </summary>
    public sealed class DMReceiverRouter
    {
        private Dictionary<Int32, IDMReceiver> CBs;
        private static DMReceiverRouter? Instance = null;

        /// <summary>
        /// GetInstance Singleton accessor
        /// </summary>
        /// <returns>the DMCallbackSingleton instance</returns>
        public static DMReceiverRouter GetInstance()
        {
            //NOTE: This is not thread safe, but since it's initialized on
            // start it should be OK. 
            if (Instance == null)
            {
                Instance = new DMReceiverRouter();
            }

            return Instance;
        }

        /// <summary>
        /// Private DMSingleton constructor, can only be called by GetInstance
        /// </summary>
        private DMReceiverRouter()
        {
            this.CBs = new Dictionary<Int32, IDMReceiver>();
        }

        /// <summary>
        /// SetCallbacks sets callbacks for a specific dm InstanceID.
        /// </summary>
        /// <param name="DMInstanceID">The DirectMessaging instance
        /// this is for</param>
        /// <param name="dmCBs">the callbacks implementation to use</param>
        /// <exception cref="Exception">exception when the interface
        /// does not exist.</exception>
        public void SetCallbacks(Int32 DMInstanceID, IDMReceiver dmCBs)
        {
            if (dmCBs == null)
            {
                throw new Exception("must implement IDMCallbackFunctions");
            }
            // NOTE: Used add here to cause a throw if duplicate key is used.
            this.CBs.Add(DMInstanceID, dmCBs);
        }

        /// <summary>
        /// Get the callbacks for a specific DM instance
        /// </summary>
        /// <param name="DMInstanceID">the dm instance we want the
        /// callbacks for.</param>
        /// <returns>the callbacks implementation for the specified
        /// dm instance.</returns>
        public IDMReceiver GetCallbacks(Int32 DMInstanceID)
        {
            return this.CBs[DMInstanceID];
        }
    }


    /// <summary>
    /// Direct Message functionality
    /// </summary>
    public class DirectMessaging
    {
        private Int32 cMixInstanceID;
        private Int32 dmInstanceID;

        /// <summary>
        /// Create a DMClient
        /// </summary>
        /// <param name="cMixInstance">cMix network interface to use</param>
        /// <param name="codenameIdentity">our Codename for DMs</param>
        /// <param name="secretPassphrase">Our codename password</param>
        /// <param name="Callbacks">Implementation of callbacks for DM</param>
        /// <returns>DM Client Instance IE</returns>
        /// <exception cref="Exception">Errors on setup</exception>
        public DirectMessaging(CMix cMixInstance,
            Byte[] codenameIdentity, String secretPassphrase,
            IDMReceiver Callbacks)
        {
            this.cMixInstanceID = cMixInstance.GetInstanceID();
            GoSlice id = NewGoSlice(codenameIdentity);
            GoString secret = NewGoString(secretPassphrase);
            cmix_dm_NewDMClient_return ret = CLIB.cmix_dm_NewDMClient(
                this.cMixInstanceID, id, secret);

            FreeGoSlice(id);
            FreeGoString(secret);

            GoError err = ret.Err;
            if (err.IsError != 0)
            {
                String errMsg = ConvertCharPtr(err.Msg, err.MsgLen);
                throw new Exception(errMsg);
            }
            this.dmInstanceID = ret.DMInstanceID;
            DMReceiverRouter cb = DMReceiverRouter.GetInstance();
            cb.SetCallbacks(this.dmInstanceID, Callbacks);
        }

        /// <summary>
        /// Get our generated DMToken, this is required to send to a
        /// Direct messaging partner. Knowing their pubkey is not enough.
        /// </summary>
        /// <returns>The DM Token for this DM Client</returns>
        /// <exception cref="Exception">Errors from Library</exception>
        public Int32 GetToken()
        {
            cmix_dm_GetDMToken_return ret = CLIB.cmix_dm_GetDMToken(
                this.dmInstanceID);
            GoError err = ret.Err;
            if (err.IsError != 0)
            {
                String errMsg = ConvertCharPtr(err.Msg, err.MsgLen);
                throw new Exception(errMsg);
            }
            return ret.Token;
        }
        /// <summary>
        /// The Codename Public Key. These are ED25519 curve keys. 
        /// </summary>
        /// <returns>Bytes of the public key</returns>
        /// <exception cref="Exception">Error from Library</exception>
        public Byte[] GetPubKey()
        {
            cmix_dm_GetDMPubKey_return ret = CLIB.cmix_dm_GetDMPubKey(
                this.dmInstanceID);
            GoError err = ret.Err;
            if (err.IsError != 0)
            {
                String errMsg = ConvertCharPtr(err.Msg, err.MsgLen);
                throw new Exception(errMsg);
            }
            return GoByteSliceToBytes(ret.PubKey);
        }

        /// <summary>
        /// SendText sends a text message type as a direct message.
        /// </summary>
        /// <param name="partnerPubKey">Public key bytes of the partner</param>
        /// <param name="dmToken">DM token of the partner</param>
        /// <param name="message">Your message</param>
        /// <param name="leaseTimeMS">How long it should stay at
        /// the receiving client.</param>
        /// <param name="cmixParamsJSON">Optional. cMix network params.
        /// Defaults should be used unless you are testing something
        /// specific.</param>
        /// <returns>A JSON Encoded SendReport</returns>
        /// <exception cref="Exception">Error on send</exception>
        public Byte[] SendText(
            Byte[] partnerPubKey, Int32 dmToken,
            String message, Int64 leaseTimeMS, Byte[] cmixParamsJSON)
        {
            GoSlice partnerKey = NewGoSlice(partnerPubKey);
            GoString goMsg = NewGoString(message);
            GoSlice cmixParams = NewGoSlice(cmixParamsJSON);
            cmix_dm_SendText_return ret = CLIB.cmix_dm_SendText(this.dmInstanceID,
                partnerKey, dmToken, goMsg, leaseTimeMS, cmixParams);

            GoError err = ret.Err;
            if (err.IsError != 0)
            {
                String errMsg = ConvertCharPtr(err.Msg, err.MsgLen);
                throw new Exception(errMsg);
            }

            FreeGoSlice(partnerKey);
            FreeGoString(goMsg);
            FreeGoSlice(cmixParams);

            Byte[] sendReport = GoByteSliceToBytes(ret.SendReportJSON);
            return sendReport;
        }

    }

    /// <summary>
    /// Setup receiver routes
    /// </summary>
    public static void SetupReceiverRouter()
    {
        DMReceiverRouterFunctions CBs = new DMReceiverRouterFunctions();
        CLIB.cmix_dm_set_router(CBs);
    }

    private static string ConvertGoString(GoString gs)
    {
        return Marshal.PtrToStringUTF8(gs.p, unchecked((int)gs.n));
    }
    private static string ConvertCharPtr(IntPtr buf, Int32 len)
    {
        return Marshal.PtrToStringUTF8(buf, len);
    }

    private static string ConvertCChar(char* buf, Int32 len)
    {
        return Marshal.PtrToStringUTF8((IntPtr)buf, len);
    }

    private static Byte[] ConvertCVoid(void* buf, Int32 len)
    {
        Int32 n = unchecked((int)(len));
        Byte[] res = new Byte[n];
        Marshal.Copy((IntPtr)buf, res, 0, n);
        return res;
    }


    private static GoString NewGoString(String newString)
    {
        // Allocate unmanaged memory for the
        // GoString
        GoString s = new GoString
        {
            p = Marshal.StringToHGlobalAnsi(newString),
            n = newString.Length
        };
        return s;
    }
    private static void FreeGoString(GoString freeMe)
    {
        Marshal.FreeHGlobal(freeMe.p);
    }
    private static GoSlice NewGoSlice(Byte[] data)
    {
        // Allocate unmanaged memory for
        // the GoSlice
        int n = data.Length;

        GoSlice gs = new GoSlice
        {
            data = Marshal.AllocHGlobal(n),
            cap = n,
            len = n
        };
        Marshal.Copy(data, 0, gs.data, n);
        return gs;
    }
    private static void FreeGoSlice(GoSlice freeMe)
    {
        Marshal.FreeHGlobal(freeMe.data);
    }
    private static Byte[] GoByteSliceToBytes(GoByteSlice slice)
    {
        Int32 n = unchecked((int)(slice.len));
        Byte[] res = new Byte[n];
        Marshal.Copy(slice.data, res, 0, n);
        return res;
    }
    private static GoByteSlice BytesToGoByteSlice(Byte[] bytes)
    {
        GoByteSlice slice = new();
        int n = bytes.Length;
        slice.len = (Int64)n;
        slice.data = Marshal.AllocHGlobal(n);
        Marshal.Copy(bytes, 0, slice.data, n);
        return slice;
    }


    /// <summary>
    /// Receive RAW direct message callback
    /// </summary>
    public delegate long DMReceiveCallbackFn(int dm_instance_id,
        void* message_id, int message_id_len,
        char* nickname, int nickname_len,
        void* text, int text_len,
        void* partnerkey, int partnerkey_len,
        void* senderkey, int senderkey_len,
        int dmToken, int codeset,
        long timestamp, long round_id, long msg_type, long status);
    /// <summary>
    /// Received Text message callback
    /// </summary>
    public delegate long DMReceiveTextCallbackFn(int dm_instance_id,
        void* mesage_id, int message_id_len,
        char* nickname, int nickname_len,
        char* text, int text_len,
        void* partnerkey, int partnerkey_len,
        void* senderkey, int senderkey_len,
        int dmToken, int codeset,
        long timestamp, long round_id, long status);
    /// <summary>
    /// Received Reply message callback
    /// </summary>
    public delegate long DMReceiveReplyCallbackFn(int dm_instance_id,
        void* mesage_id, int message_id_len,
        void* reply_to, int reply_to_len,
        char* nickname, int nickname_len,
        char* text, int text_len,
        void* partnerkey, int partnerkey_len,
        void* senderkey, int senderkey_len,
        int dmToken, int codeset,
        long timestamp, long round_id, long status);
    /// <summary>
    /// Received Reaction message callback
    /// </summary>
    public delegate long DMReceiveReactionCallbackFn(int dm_instance_id,
        void* mesage_id, int message_id_len,
        void* reaction_to, int reaction_to_len,
        char* nickname, int nickname_len,
        char* text, int text_len,
        void* partnerkey, int partnerkey_len,
        void* senderkey, int senderkey_len,
        int dmToken, int codeset,
        long timestamp, long round_id, long status);
    /// <summary>
    /// Message was updated callback. Used to tell UI progress as
    /// message is sent through the network. 
    /// </summary>
    public delegate long DMUpdateSentStatusCallbackFn(int dm_instance_id,
        long uuid,
        void* message_id, int message_id_len, long timestamp,
        long round_id, long status);

    /// <summary>
    /// User is blocked callback. Used to tell UI a user is blocked.
    /// </summary>
    public delegate void DMBlockUserCallbackFn(int dm_instance_id,
        void* pubkey, int pubkey_len);

    /// <summary>
    /// User is unblocked callback. Used to tell UI a user is unblocked.
    /// </summary>
    public delegate void DMUnblockUserCallbackFn(int dm_instance_id,
        void* pubkey, int pubkey_len);

    /// <summary>
    /// GetConversation callback. Used to retrieve conversation object.
    /// </summary>
    public delegate GoByteSlice DMGetConversationCallbackFn(int dm_instance_id,
        void* senderkey, int senderkey_len);

    /// <summary>
    /// GetConversations callback. Used to retrieve all conversation objects.
    /// </summary>
    public delegate GoByteSlice DMGetConversationsCallbackFn(
        int dm_instance_id);

    /// <summary>
    /// DeleteMessage callback Used to signal a delete message request
    /// </summary>
    public delegate int DMDeleteMessageCallbackFn(int dm_instance_id,
        void* message_id, int message_id_len,
        void* pubkey, int pubkey_len);

    /// <summary>
    /// DMEventupdate callback for events
    /// </summary>
    public delegate void DMEventUpdateCallbackFn(int dm_instance_id,
        long event_type, void* json_data,
        int json_data_len);

    /// <summary>
    /// Pass through implementation for C Library Callback for
    /// DMReceive
    /// </summary>
    public static long DMReceive(int dm_instance_id,
        void* message_id, int message_id_len,
        char* nickname, int nickname_len,
        void* text, int text_len,
        void* partnerkey, int partnerkey_len,
        void* senderkey, int senderkey_len,
        int dmToken, int codeset,
        long timestamp, long round_id, long msg_type, long status)
    {
        DMReceiverRouter dm = DMReceiverRouter.GetInstance();
        IDMReceiver cbs = dm.GetCallbacks(dm_instance_id);

        Byte[] MsgID = ConvertCVoid(message_id, message_id_len);
        String Nick = ConvertCChar(nickname, nickname_len);
        Byte[] Text = ConvertCVoid(text, text_len);
        Byte[] partnerKey = ConvertCVoid(partnerkey, partnerkey_len);
        Byte[] senderKey = ConvertCVoid(senderkey, senderkey_len);

        Console.WriteLine("DMReceive: {0}, {1}: {2}",
            System.Convert.ToBase64String(partnerKey),
            System.Convert.ToBase64String(senderKey), dmToken, Text);

        return cbs.Receive(MsgID,
            Nick, Text, partnerKey, senderKey, dmToken, codeset, timestamp,
            round_id, msg_type, status);
    }
    /// <summary>
    /// Pass through implementation for C Library Callback for
    /// DMReceiveText
    /// </summary>
    public static long DMReceiveText(int dm_instance_id,
        void* message_id, int message_id_len,
        char* nickname, int nickname_len,
        char* text, int text_len,
        void* partnerkey, int partnerkey_len,
        void* senderkey, int senderkey_len,
        int dmToken, int codeset,
        long timestamp, long round_id, long status)
    {
        DMReceiverRouter dm = DMReceiverRouter.GetInstance();
        IDMReceiver cbs = dm.GetCallbacks(dm_instance_id);

        Byte[] MsgID = ConvertCVoid(message_id, message_id_len);
        String Nick = ConvertCChar(nickname, nickname_len);
        String Text = ConvertCChar(text, text_len);
        Byte[] partnerKey = ConvertCVoid(partnerkey, partnerkey_len);
        Byte[] senderKey = ConvertCVoid(senderkey, senderkey_len);

        Console.WriteLine("DMReceiveText: {0}->{1}, {2}: {3}",
            System.Convert.ToBase64String(partnerKey),
            System.Convert.ToBase64String(senderKey), dmToken, Text);

        return cbs.ReceiveText(MsgID,
            Nick, Text, partnerKey, senderKey, dmToken, codeset, timestamp,
            round_id, status);
    }
    /// <summary>
    /// Pass through implementation for C Library Callback for
    /// DMReceiveReply
    /// </summary>
    public static long DMReceiveReply(int dm_instance_id,
        void* message_id, int message_id_len,
        void* reply_to, int reply_to_len,
        char* nickname, int nickname_len,
        char* text, int text_len,
        void* partnerkey, int partnerkey_len,
        void* senderkey, int senderkey_len,
        int dmToken, int codeset,
        long timestamp, long round_id, long status)
    {
        DMReceiverRouter dm = DMReceiverRouter.GetInstance();
        IDMReceiver cbs = dm.GetCallbacks(dm_instance_id);

        Byte[] MsgID = ConvertCVoid(message_id, message_id_len);
        Byte[] ReplyTo = ConvertCVoid(reply_to, reply_to_len);
        String Nick = ConvertCChar(nickname, nickname_len);
        String Text = ConvertCChar(text, text_len);
        Byte[] partnerKey = ConvertCVoid(partnerkey, partnerkey_len);
        Byte[] senderKey = ConvertCVoid(senderkey, senderkey_len);

        Console.WriteLine("DMReceiveReply {0}->{1}, {2}: {3}",
            System.Convert.ToBase64String(partnerKey),
            System.Convert.ToBase64String(senderKey), dmToken, Text);

        return cbs.ReceiveReply(MsgID, ReplyTo,
            Nick, Text, partnerKey, senderKey, dmToken, codeset, timestamp,
            round_id, status);
    }
    /// <summary>
    /// Pass through implementation for C Library Callback for
    /// DMReceiveReaction
    /// </summary>
    public static long DMReceiveReaction(int dm_instance_id,
        void* message_id, int message_id_len,
        void* reaction_to, int reaction_to_len,
        char* nickname, int nickname_len,
        char* text, int text_len,
        void* partnerkey, int partnerkey_len,
        void* senderkey, int senderkey_len,
        int dmToken, int codeset,
        long timestamp, long round_id, long status)
    {
        DMReceiverRouter dm = DMReceiverRouter.GetInstance();
        IDMReceiver cbs = dm.GetCallbacks(dm_instance_id);

        Byte[] MsgID = ConvertCVoid(message_id, message_id_len);
        Byte[] ReactionTo = ConvertCVoid(reaction_to, reaction_to_len);
        String Nick = ConvertCChar(nickname, nickname_len);
        String Text = ConvertCChar(text, text_len);
        Byte[] partnerKey = ConvertCVoid(partnerkey, partnerkey_len);
        Byte[] senderKey = ConvertCVoid(senderkey, senderkey_len);

        Console.WriteLine("DMReceiveReaction {0}->{1}, {2}: {3}",
            System.Convert.ToBase64String(partnerKey),
            System.Convert.ToBase64String(senderKey), dmToken, Text);

        return cbs.ReceiveReaction(MsgID, ReactionTo,
            Nick, Text, partnerKey, senderKey, dmToken, codeset, timestamp,
            round_id, status);
    }
    /// <summary>
    /// Pass through implementation for C Library Callback for
    /// DMUpdateSentStatus
    /// </summary>
    public static long DMUpdateSentStatus(int dm_instance_id,
        long uuid,
        void* message_id, int message_id_len, long timestamp,
        long round_id, long status)
    {
        DMReceiverRouter dm = DMReceiverRouter.GetInstance();
        IDMReceiver cbs = dm.GetCallbacks(dm_instance_id);

        Byte[] MsgID = ConvertCVoid(message_id, message_id_len);
        Console.WriteLine("DMUpdateSentStatus {0}: {1}",
            System.Convert.ToBase64String(MsgID), status);

        return cbs.UpdateSentStatus(uuid, MsgID,
            timestamp, round_id, status);
    }
    /// <summary>
    /// User is blocked callback. Used to tell UI a user is blocked.
    /// </summary>
    public static void DMBlockUser(int dm_instance_id,
        void* pubkey, int pubkey_len)
    {
        DMReceiverRouter dm = DMReceiverRouter.GetInstance();
        IDMReceiver cbs = dm.GetCallbacks(dm_instance_id);

        Byte[] publicKey = ConvertCVoid(pubkey, pubkey_len);
        Console.WriteLine("DMBlockUser {0}",
            System.Convert.ToBase64String(publicKey));

        cbs.BlockUser(publicKey);
    }

    /// <summary>
    /// User is unblocked callback. Used to tell UI a user is unblocked.
    /// </summary>
    public static void DMUnblockUser(int dm_instance_id,
        void* pubkey, int pubkey_len)
    {
        DMReceiverRouter dm = DMReceiverRouter.GetInstance();
        IDMReceiver cbs = dm.GetCallbacks(dm_instance_id);

        Byte[] publicKey = ConvertCVoid(pubkey, pubkey_len);
        Console.WriteLine("DMUnblockUser {0}",
            System.Convert.ToBase64String(publicKey));

        cbs.UnblockUser(publicKey);
    }

    /// <summary>
    /// GetConversation callback. Used to retrieve conversation object.
    /// </summary>
    public static GoByteSlice DMGetConversation(int dm_instance_id,
        void* senderkey, int senderkey_len)
    {
        DMReceiverRouter dm = DMReceiverRouter.GetInstance();
        IDMReceiver cbs = dm.GetCallbacks(dm_instance_id);

        Byte[] publicKey = ConvertCVoid(senderkey, senderkey_len);
        Console.WriteLine("DMGetConversation {0}",
            System.Convert.ToBase64String(publicKey));

        Byte[] retval = cbs.GetConversation(publicKey);

        return BytesToGoByteSlice(retval);
    }

    /// <summary>
    /// GetConversations callback. Used to retrieve all conversation objects.
    /// </summary>
    public static GoByteSlice DMGetConversations(int dm_instance_id)
    {
        DMReceiverRouter dm = DMReceiverRouter.GetInstance();
        IDMReceiver cbs = dm.GetCallbacks(dm_instance_id);

        Console.WriteLine("DMGetConversations");

        Byte[] retval = cbs.GetConversations();

        return BytesToGoByteSlice(retval);
    }

    /// <summary>
    /// DeleteMessage callback Used to signal a delete message request
    /// </summary>
    public static int DMDeleteMessage(int dm_instance_id,
        void* message_id, int message_id_len, void* pubkey, int pubkey_len)
    {
        DMReceiverRouter dm = DMReceiverRouter.GetInstance();
        IDMReceiver cbs = dm.GetCallbacks(dm_instance_id);

        Console.WriteLine("DMDeleteMessage");

        Byte[] messageID = ConvertCVoid(message_id, message_id_len);
        Console.WriteLine("DMDeleteMessage messageID {0}",
            System.Convert.ToBase64String(messageID));
        Byte[] publicKey = ConvertCVoid(pubkey, pubkey_len);
        Console.WriteLine("DMDeleteMessage pubkey {0}",
            System.Convert.ToBase64String(publicKey));


        if (cbs.DeleteMessage(messageID, publicKey))
        {
            return 1;
        }
        return 0;
    }

    /// <summary>
    /// DMEventupdate callback for events
    /// </summary>
    public static void DMEventUpdate(int dm_instance_id, long event_type,
        void* json_data, int json_data_len)
    {
        DMReceiverRouter dm = DMReceiverRouter.GetInstance();
        IDMReceiver cbs = dm.GetCallbacks(dm_instance_id);

        Console.WriteLine("DMEventUpdate");

        Byte[] jsonData = ConvertCVoid(json_data, json_data_len);
        Console.WriteLine("DMDeleteMessage json_data {0}",
            System.Convert.ToBase64String(jsonData));

        cbs.EventUpdate(event_type, jsonData);
    }


    /// <summary>
    /// DMReceiverRouterFunctions holds the function pointers
    /// to the various DMReceiver reception functions for direct messages.
    /// You must create this structure and call cmix_dm_set_router before
    /// you can receive messages from xx network cMix.
    /// </summary>
    [StructLayout(LayoutKind.Sequential)]
    public class DMReceiverRouterFunctions
    {
        DMReceiveCallbackFn receiveFn = DMReceive;
        DMReceiveTextCallbackFn receiveTextFn = DMReceiveText;
        DMReceiveReplyCallbackFn receiveReplyFn = DMReceiveReply;
        DMReceiveReactionCallbackFn receiveReactionFn = DMReceiveReaction;
        DMUpdateSentStatusCallbackFn updateSentStatusFn = DMUpdateSentStatus;
        DMBlockUserCallbackFn blockUserFn = DMBlockUser;
        DMUnblockUserCallbackFn unblockUserFn = DMUnblockUser;
        DMGetConversationCallbackFn getConversationFn = DMGetConversation;
        DMGetConversationsCallbackFn getConversationsFn = DMGetConversations;
        DMDeleteMessageCallbackFn deleteMessageFn = DMDeleteMessage;
        DMEventUpdateCallbackFn eventUpdatefn = DMEventUpdate;
    }

    /// <summary>
    /// Internal CLIB function mappings. Do not use directly.
    /// </summary>
    internal static class CLIB
    {
        private const string xxdkLib = "xxdk";

        static CLIB()
        {
            // Setup our custom import resolver to search the runtimes directory
            NativeLibrary.SetDllImportResolver(typeof(CLIB).Assembly,
                DllImportResolver);
        }

        static readonly bool IsWindows = RuntimeInformation.IsOSPlatform(OSPlatform.Windows);
        static readonly bool IsMac = RuntimeInformation.IsOSPlatform(OSPlatform.OSX);
        static readonly bool IsLinux = RuntimeInformation.IsOSPlatform(OSPlatform.Linux);

        static readonly bool isX64 = (RuntimeInformation.ProcessArchitecture == Architecture.X64) ? true : false;
        static readonly bool isArm64 = (RuntimeInformation.ProcessArchitecture == Architecture.Arm64) ? true : false;

        private static IntPtr DllImportResolver(string libraryName,
            Assembly assembly, DllImportSearchPath? searchPath)
        {
            if (libraryName != xxdkLib)
            {
                //Console.Error.WriteLine("Library Name: " + libraryName);
                return IntPtr.Zero;
            }

            string myOS;
            string myLibName = libraryName;
            if (IsWindows) {
                myOS = "win";
                myLibName = myLibName + ".dll";
            } else if (IsMac) {
                myOS = "darwin";
                myLibName = "lib" + myLibName + ".so";
            }
            else if (IsLinux) {
                myOS = "linux";
                myLibName = "lib" + myLibName + ".so";
            }
            else
            {
                throw new Exception("unsupported operating system: "
                    + RuntimeInformation.OSDescription);
            }

            string myARCH;
            if (isX64)
            {
                myARCH = "x64";
            }
            else if (isArm64)
            {
                myARCH = "arm64";
            }
            else
            {
                throw new Exception("unsupported architecture: "
                    + RuntimeInformation.ProcessArchitecture);
            }

            string myTarget = myOS + "-" + myARCH;
            string mySearchPath = Path.Join("runtimes", myTarget, "native");

            object? nativeSearchContexts = AppContext.GetData(
                "PLATFORM_RESOURCE_ROOTS");
            var delimiter = RuntimeInformation.IsOSPlatform(
                OSPlatform.Windows) ? ";" : ":";
            if (nativeSearchContexts != null)
            {
                string nativePaths = (string)nativeSearchContexts;
                foreach (var directory in nativePaths.Split(delimiter))
                {
                    var path = Path.Combine(directory, mySearchPath,
                        myLibName);
                    //Console.Error.WriteLine("PATH: " + path);
                    if (Path.Exists(path))
                    {
                        return NativeLibrary.Load(path);
                    }
                }
            }

            // Next, try to load any OS-provided version of the library
            IntPtr lib;
            if (NativeLibrary.TryLoad(myLibName, out lib))
            {
                return lib;
            }
            return NativeLibrary.Load(myLibName, assembly, searchPath);
        }

        // This function sets the cMix DM Receiver callbacks in the library
        // You must call it before starting networking and receiving
        // messages.
        [DllImport(xxdkLib)]
        public static extern void cmix_dm_set_router(
            DMReceiverRouterFunctions cbs);

        // GetVersion returns the xxdk.SEMVER.
        [DllImport(xxdkLib)]
        public static extern GoString GetVersion();

        // GetGitVersion returns the xxdk.GITVERSION.
        [DllImport(xxdkLib)]
        public static extern GoString GetGitVersion();

        // GetDependencies returns the xxdk.DEPENDENCIES.
        [DllImport(xxdkLib)]
        public static extern GoString GetDependencies();

        // NewCmix creates user storage, generates keys, connects, and registers with
        // the network. Note that this does not register a username/identity, but merely
        // creates a new cryptographic identity for adding such information at a later
        // date.
        //
        // Users of this function should delete the storage directory on error.
        [DllImport(xxdkLib)]
        public static extern GoError NewCmix(GoString ndfJSON,
            GoString storageDir, GoSlice password,
            GoString registrationCode);

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
        [DllImport(xxdkLib)]
        public static extern LoadCmix_return LoadCmix(GoString storageDir,
            GoSlice password, GoSlice cmixParamsJSON);


        // cmix_GetReceptionID returns the current default reception ID
        [DllImport(xxdkLib)]
        public static extern cmix_GetReceptionID_return cmix_GetReceptionID(
            GoInt32 cMixInstanceID);
        [DllImport(xxdkLib)]
        public static extern cmix_EKVGet_return cmix_EKVGet(
            GoInt32 cMixInstanceID, GoString key);
        [DllImport(xxdkLib)]
        public static extern GoError cmix_EKVSet(GoInt32 cMixInstanceID,
            GoString key, GoSlice value);
        [DllImport(xxdkLib)]
        public static extern GoError cmix_StartNetworkFollower(
            GoInt32 cMixInstanceID, GoInt timeoutMS);
        [DllImport(xxdkLib)]
        public static extern GoError cmix_StopNetworkFollower(
            GoInt32 cMixInstanceID);
        [DllImport(xxdkLib)]
        public static extern GoError cmix_WaitForNetwork(GoInt32 cMixInstanceID,
            GoInt timeoutMS);
        [DllImport(xxdkLib)]
        public static extern GoUint8 cmix_ReadyToSend(GoInt32 cMixInstanceID);
        [DllImport(xxdkLib)]
        public static extern GoByteSlice cmix_GenerateCodenameIdentity(
            GoString secretPassphrase);
        [DllImport(xxdkLib)]
        public static extern cmix_dm_NewDMClient_return cmix_dm_NewDMClient(
            GoInt32 cMixInstanceID, GoSlice codenameIdentity,
            GoString secretPassphrase);
        [DllImport(xxdkLib)]
        public static extern cmix_dm_GetDMToken_return cmix_dm_GetDMToken(
            GoInt32 dmInstanceID);
        [DllImport(xxdkLib)]
        public static extern cmix_dm_GetDMPubKey_return cmix_dm_GetDMPubKey(
            GoInt32 dmInstanceID);
        [DllImport(xxdkLib)]
        public static extern cmix_dm_SendText_return cmix_dm_SendText(
            GoInt32 dmInstanceID, GoSlice partnerPubKey, GoInt32 dmToken,
            GoString message, GoInt64 leaseTimeMS, GoSlice cmixParamsJSON);

    }

}
