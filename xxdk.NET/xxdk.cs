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
[StructLayout(LayoutKind.Sequential)]
struct GoByteSlice
{
    public Int64 len;
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
    public GoUint32 Token;
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
    public static class CMix
    {
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
        public static int LoadCmix(String storageDir, Byte[] password,
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

            return ret.cMixInstanceID;
        }

        /// <summary>
        /// cmix_GetReceptionID returns the current default reception ID
        /// </summary>
        /// <param name="cMixInstanceID">Instance ID from LoadCmix</param>
        /// <returns>The cMix Reception ID, used to receive RAW cMix Messages</returns>
        /// <exception cref="Exception">Error occured in library</exception>
        public static Byte[] GetReceptionID(Int32 cMixInstanceID)
        {
            cmix_GetReceptionID_return rid = CLIB.cmix_GetReceptionID(
                cMixInstanceID);
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
        /// <param name="cMixInstanceID">Instance ID for cMix</param>
        /// <param name="key">The key associated with the value</param>
        /// <returns>Bytes of the value</returns>
        /// <exception cref="Exception">Error or if key cannot be found</exception>
        public static Byte[] EKVGet(Int32 cMixInstanceID, String key)
        {
            GoString goKey = NewGoString(key);
            cmix_EKVGet_return ret = CLIB.cmix_EKVGet(cMixInstanceID, goKey);

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
        /// <param name="cMixInstanceID">Instance ID for cMix</param>
        /// <param name="key">The key associated with the value</param>
        /// <param name="value">The bytes to store as the value</param>
        /// <exception cref="Exception">Write errors</exception>
        public static void EKVSet(Int32 cMixInstanceID, String key,
            Byte[] value)
        {
            GoString goKey = NewGoString(key);
            GoSlice goVal = NewGoSlice(value);
            GoError err = CLIB.cmix_EKVSet(cMixInstanceID, goKey, goVal);
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
        /// <param name="cMixInstanceID">Instance ID for cMix</param>
        /// <param name="timeoutMS">Error out if we haven't connected
        /// by this many milliseconds</param>
        /// <exception cref="Exception">Timeout or other error on
        /// connection</exception>
        public static void StartNetworkFollower(Int32 cMixInstanceID,
            Int32 timeoutMS)
        {
            GoError err = CLIB.cmix_StartNetworkFollower(cMixInstanceID,
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
        /// <param name="cMixInstanceID">Instance ID for cMix</param>
        /// <exception cref="Exception">Timeout or other error on
        /// shutdown</exception>
        public static void StopNetworkFollower(Int32 cMixInstanceID)
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
        /// <param name="cMixInstanceID">Instance Id for cMix</param>
        /// <param name="timeoutMS">Timeout in milliseconds</param>
        /// <exception cref="Exception">Timed out or other send
        /// error</exception>
        public static void WaitForNetwork(Int32 cMixInstanceID,
            Int32 timeoutMS)
        {
            GoError err = CLIB.cmix_WaitForNetwork(cMixInstanceID, timeoutMS);
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
        /// <param name="cMixInstanceID">cMix Instance ID</param>
        /// <returns>If we are connected</returns>
        public static Boolean ReadyToSend(Int32 cMixInstanceID)
        {
            GoUint8 ready = CLIB.cmix_ReadyToSend(cMixInstanceID);
            if (ready != 0)
            {
                return true;
            }
            return false;
        }
    }

    /// <summary>
    /// Direct Message functionality
    /// </summary>
    public static class DirectMessaging
    {
        /// <summary>
        /// Create a DMClient
        /// </summary>
        /// <param name="cMixInstanceID">The cMix Instance Id to use</param>
        /// <param name="codenameIdentity">our Codename for DMs</param>
        /// <param name="secretPassphrase">Our codename password</param>
        /// <returns>DM Client Instance IE</returns>
        /// <exception cref="Exception">Errors on setup</exception>
        public static Int32 NewClient(Int32 cMixInstanceID,
            Byte[] codenameIdentity, String secretPassphrase)
        {
            GoSlice id = NewGoSlice(codenameIdentity);
            GoString secret = NewGoString(secretPassphrase);
            cmix_dm_NewDMClient_return ret = CLIB.cmix_dm_NewDMClient(
                cMixInstanceID, id, secret);

            FreeGoSlice(id);
            FreeGoString(secret);

            GoError err = ret.Err;
            if (err.IsError != 0)
            {
                String errMsg = ConvertCharPtr(err.Msg, err.MsgLen);
                throw new Exception(errMsg);
            }
            return ret.DMInstanceID;
        }

        /// <summary>
        /// Get our generated DMToken, this is required to send to a
        /// Direct messaging partner. Knowing their pubkey is not enough.
        /// </summary>
        /// <param name="dmInstanceID">DM Client ID</param>
        /// <returns>The DM Token for this DM Client</returns>
        /// <exception cref="Exception">Errors from Library</exception>
        public static UInt32 GetToken(Int32 dmInstanceID)
        {
            cmix_dm_GetDMToken_return ret = CLIB.cmix_dm_GetDMToken(
                dmInstanceID);
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
        /// <param name="dmInstanceID">DM Client ID</param>
        /// <returns>Bytes of the public key</returns>
        /// <exception cref="Exception">Error from Library</exception>
        public static Byte[] GetPubKey(Int32 dmInstanceID)
        {
            cmix_dm_GetDMPubKey_return ret = CLIB.cmix_dm_GetDMPubKey(
                dmInstanceID);
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
        /// <param name="dmInstanceID">DM Client ID</param>
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
        public static Byte[] SendText(Int32 dmInstanceID,
            Byte[] partnerPubKey, UInt32 dmToken,
            String message, Int64 leaseTimeMS, Byte[] cmixParamsJSON)
        {
            GoSlice partnerKey = NewGoSlice(partnerPubKey);
            GoString goMsg = NewGoString(message);
            GoSlice cmixParams = NewGoSlice(cmixParamsJSON);
            cmix_dm_SendText_return ret = CLIB.cmix_dm_SendText(dmInstanceID,
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
    /// Setup receiver callbacks (FIXME).
    /// </summary>
    public static void SetupReceiverCallbacks()
    {
        DMReceiverCallbackFunctions CBs = new DMReceiverCallbackFunctions();
        CLIB.cmix_dm_set_callbacks(CBs);
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

    /// <summary>
    /// Receive RAW direct message callback
    /// </summary>
    public delegate long DMReceiveCallbackFn(int dm_instance_id,
        void* message_id, int message_id_len,
        char* nickname, int nickname_len,
        void* text, int text_len,
        void* partnerkey, int partnerkey_len,
        void* senderkey, int senderkey_len,
        uint dmToken, int codeset,
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
        uint dmToken, int codeset,
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
        uint dmToken, int codeset,
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
        uint dmToken, int codeset,
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
    /// Dummy implementation of callback
    /// </summary>
    public static long DMReceive(int dm_instance_id,
        void* message_id, int message_id_len,
        char* nickname, int nickname_len,
        void* text, int text_len,
        void* partnerkey, int partnerkey_len,
        void* senderkey, int senderkey_len,
        uint dmToken, int codeset,
        long timestamp, long round_id, long msg_type, long status)
    {
        Byte[] partnerKey = ConvertCVoid(partnerkey, partnerkey_len);
        Byte[] senderKey = ConvertCVoid(senderkey, senderkey_len);
        Byte[] Message = ConvertCVoid(text, text_len);
        Console.WriteLine("DMReceive: { 0}, { 1}: { 2}",
            System.Convert.ToBase64String(partnerKey), System.Convert.ToBase64String(senderKey), dmToken, Message);
        return 0;
    }
    /// <summary>
    /// Dummy implementation of callback
    /// </summary>
    public static long DMReceiveText(int dm_instance_id,
        void* mesage_id, int message_id_len,
        char* nickname, int nickname_len,
        char* text, int text_len,
        void* partnerkey, int partnerkey_len,
        void* senderkey, int senderkey_len,
        uint dmToken, int codeset,
        long timestamp, long round_id, long status)
    {
        Byte[] partnerKey = ConvertCVoid(partnerkey, partnerkey_len);
        Byte[] senderKey = ConvertCVoid(senderkey, senderkey_len);
        String Message = ConvertCChar(text, text_len);
        Console.WriteLine("DMReceiveText: {0}->{1}, {2}: {3}",
            System.Convert.ToBase64String(partnerKey),
            System.Convert.ToBase64String(senderKey), dmToken, Message);
        return 0;
    }
    /// <summary>
    /// Dummy implementation of callback
    /// </summary>
    public static long DMReceiveReply(int dm_instance_id,
        void* mesage_id, int message_id_len,
        void* reply_to, int reply_to_len,
        char* nickname, int nickname_len,
        char* text, int text_len,
        void* partnerkey, int partnerkey_len,
        void* senderkey, int senderkey_len,
        uint dmToken, int codeset,
        long timestamp, long round_id, long status)
    {
        Byte[] partnerKey = ConvertCVoid(partnerkey, partnerkey_len);
        Byte[] senderKey = ConvertCVoid(senderkey, senderkey_len);
        String Message = ConvertCChar(text, text_len);
        Console.WriteLine("DMReceiveReply {0}->{1}, {2}: {3}",
            System.Convert.ToBase64String(partnerKey),
            System.Convert.ToBase64String(senderKey), dmToken, Message);
        return 0;
    }
    /// <summary>
    /// Dummy implementation of callback
    /// </summary>
    public static long DMReceiveReaction(int dm_instance_id,
        void* mesage_id, int message_id_len,
        void* reaction_to, int reaction_to_len,
        char* nickname, int nickname_len,
        char* text, int text_len,
        void* partnerkey, int partnerkey_len,
        void* senderkey, int senderkey_len,
        uint dmToken, int codeset,
        long timestamp, long round_id, long status)
    {
        Byte[] partnerKey = ConvertCVoid(partnerkey, partnerkey_len);
        Byte[] senderKey = ConvertCVoid(senderkey, senderkey_len);
        String Message = ConvertCChar(text, text_len);
        Console.WriteLine("DMReceiveReaction {0}->{1}, {2}: {3}",
            System.Convert.ToBase64String(partnerKey),
            System.Convert.ToBase64String(senderKey), dmToken, Message);
        return 0;
    }
    /// <summary>
    /// Dummy implementation of callback
    /// </summary>
    public static long DMUpdateSentStatus(int dm_instance_id,
        long uuid,
        void* message_id, int message_id_len, long timestamp,
        long round_id, long status)
    {
        Byte[] MsgID = ConvertCVoid(message_id, message_id_len);
        Console.WriteLine("DMUpdateSentStatus {0}: {1}",
            System.Convert.ToBase64String(MsgID), status);
        return 0;
    }

    /// <summary>
    /// DMReceiverCallbackFunctions holds the callback function pointers
    /// to the various reception functions for direct messages. You must
    /// Create this structure and call cmix_dm_set_callbacks before
    /// you can receive messages from xx network cMix.
    /// </summary>
    [StructLayout(LayoutKind.Sequential)]
    public class DMReceiverCallbackFunctions
    {
        DMReceiveCallbackFn receiveFn = DMReceive;
        DMReceiveTextCallbackFn receiveTextFn = DMReceiveText;
        DMReceiveReplyCallbackFn receiveReplyFn = DMReceiveReply;
        DMReceiveReactionCallbackFn receiveReactionFn = DMReceiveReaction;
        DMUpdateSentStatusCallbackFn updateSentStatusFn = DMUpdateSentStatus;
    }

    /// <summary>
    /// Internal CLIB function mappings. Do not use directly.
    /// </summary>
    static class CLIB
    {
        // This function sets the cMix DM Receiver callbacks in the library
        // You must call it before starting networking and receiving
        // messages.
        [DllImport("libxxdk.so")]
        public static extern void cmix_dm_set_callbacks(
            DMReceiverCallbackFunctions cbs);

        // GetVersion returns the xxdk.SEMVER.
        [DllImport("libxxdk.so")]
        public static extern GoString GetVersion();

        // GetGitVersion returns the xxdk.GITVERSION.
        [DllImport("libxxdk.so")]
        public static extern GoString GetGitVersion();

        // GetDependencies returns the xxdk.DEPENDENCIES.
        [DllImport("libxxdk.so")]
        public static extern GoString GetDependencies();

        // NewCmix creates user storage, generates keys, connects, and registers with
        // the network. Note that this does not register a username/identity, but merely
        // creates a new cryptographic identity for adding such information at a later
        // date.
        //
        // Users of this function should delete the storage directory on error.
        [DllImport("libxxdk.so")]
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
        [DllImport("libxxdk.so")]
        public static extern LoadCmix_return LoadCmix(GoString storageDir,
            GoSlice password, GoSlice cmixParamsJSON);


        // cmix_GetReceptionID returns the current default reception ID
        [DllImport("libxxdk.so")]
        public static extern cmix_GetReceptionID_return cmix_GetReceptionID(
            GoInt32 cMixInstanceID);
        [DllImport("libxxdk.so")]
        public static extern cmix_EKVGet_return cmix_EKVGet(
            GoInt32 cMixInstanceID, GoString key);
        [DllImport("libxxdk.so")]
        public static extern GoError cmix_EKVSet(GoInt32 cMixInstanceID,
            GoString key, GoSlice value);
        [DllImport("libxxdk.so")]
        public static extern GoError cmix_StartNetworkFollower(
            GoInt32 cMixInstanceID, GoInt timeoutMS);
        [DllImport("libxxdk.so")]
        public static extern GoError cmix_StopNetworkFollower(
            GoInt32 cMixInstanceID);
        [DllImport("libxxdk.so")]
        public static extern GoError cmix_WaitForNetwork(GoInt32 cMixInstanceID,
            GoInt timeoutMS);
        [DllImport("libxxdk.so")]
        public static extern GoUint8 cmix_ReadyToSend(GoInt32 cMixInstanceID);
        [DllImport("libxxdk.so")]
        public static extern GoByteSlice cmix_GenerateCodenameIdentity(
            GoString secretPassphrase);
        [DllImport("libxxdk.so")]
        public static extern cmix_dm_NewDMClient_return cmix_dm_NewDMClient(
            GoInt32 cMixInstanceID, GoSlice codenameIdentity,
            GoString secretPassphrase);
        [DllImport("libxxdk.so")]
        public static extern cmix_dm_GetDMToken_return cmix_dm_GetDMToken(
            GoInt32 dmInstanceID);
        [DllImport("libxxdk.so")]
        public static extern cmix_dm_GetDMPubKey_return cmix_dm_GetDMPubKey(
            GoInt32 dmInstanceID);
        [DllImport("libxxdk.so")]
        public static extern cmix_dm_SendText_return cmix_dm_SendText(
            GoInt32 dmInstanceID, GoSlice partnerPubKey, GoUint32 dmToken,
            GoString message, GoInt64 leaseTimeMS, GoSlice cmixParamsJSON);

    }

}