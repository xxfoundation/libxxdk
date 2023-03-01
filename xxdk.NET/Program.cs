using System;
using System.Buffers.Text;
using System.Diagnostics.Metrics;
using System.Net.Sockets;
using System.Runtime.InteropServices;
using System.Runtime.Intrinsics.X86;
using System.Security.Principal;
using System.Text;
using System.CommandLine;

namespace xxDK;

/// <summary>
/// Go Data Structures
/// </summary>
///
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

public unsafe class Program
{
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
    /// Callback Functions to implement.
    /// </summary>
    public delegate long DMReceiveCallbackFn(int dm_instance_id,
        void* message_id, int message_id_len,
        char* nickname, int nickname_len,
        void* text, int text_len,
        void* pubkey, int pubkey_len,
        int dmToken, int codeset,
        long timestamp, long round_id, long msg_type, long status);
    public delegate long DMReceiveTextCallbackFn(int dm_instance_id,
        void* mesage_id, int message_id_len,
        char* nickname, int nickname_len,
        char* text, int text_len,
        void* pubkey, int pubkey_len,
        int dmToken, int codeset,
        long timestamp, long round_id, long status);
    public delegate long DMReceiveReplyCallbackFn(int dm_instance_id,
        void* mesage_id, int message_id_len,
        void* reply_to, int reply_to_len,
        char* nickname, int nickname_len,
        char* text, int text_len,
        void* pubkey, int pubkey_len,
        int dmToken, int codeset,
        long timestamp, long round_id, long status);
    public delegate long DMReceiveReactionCallbackFn(int dm_instance_id,
        void* mesage_id, int message_id_len,
        void* reaction_to, int reaction_to_len,
        char* nickname, int nickname_len,
        char* text, int text_len,
        void* pubkey, int pubkey_len,
        int dmToken, int codeset,
        long timestamp, long round_id, long status);
    public delegate long DMUpdateSentStatusCallbackFn(int dm_instance_id,
        long uuid,
        void* message_id, int message_id_len, long timestamp,
        long round_id, long status);

    public static long DMReceive(int dm_instance_id,
        void* message_id, int message_id_len,
        char* nickname, int nickname_len,
        void* text, int text_len,
        void* pubkey, int pubkey_len,
        int dmToken, int codeset,
        long timestamp, long round_id, long msg_type, long status)
    {
        Console.WriteLine("DMReceive");
        return 0;
    }
    public static long DMReceiveText(int dm_instance_id,
        void* mesage_id, int message_id_len,
        char* nickname, int nickname_len,
        char* text, int text_len,
        void* pubkey, int pubkey_len,
        int dmToken, int codeset,
        long timestamp, long round_id, long status)
    {
        Console.WriteLine("DMReceiveText");
        return 0;
    }
    public static long DMReceiveReply(int dm_instance_id,
        void* mesage_id, int message_id_len,
        void* reply_to, int reply_to_len,
        char* nickname, int nickname_len,
        char* text, int text_len,
        void* pubkey, int pubkey_len,
        int dmToken, int codeset,
        long timestamp, long round_id, long status)
    {
        Console.WriteLine("DMReceiveReply");
        return 0;
    }
    public static long DMReceiveReaction(int dm_instance_id,
        void* mesage_id, int message_id_len,
        void* reaction_to, int reaction_to_len,
        char* nickname, int nickname_len,
        char* text, int text_len,
        void* pubkey, int pubkey_len,
        int dmToken, int codeset,
        long timestamp, long round_id, long status)
    {
        Console.WriteLine("DMReceiveReaction");
        return 0;
    }
    public static long DMUpdateSentStatus(int dm_instance_id,
        long uuid,
        void* message_id, int message_id_len, long timestamp,
        long round_id, long status)
    {
        Console.WriteLine("DMUpdateSentStatus");
        return 0;
    }
    // DMReceiverCallbackFunctions holds the callback function pointers
    // to the various reception functions for direct messages. You must
    // Create this structure and call cmix_dm_set_callbacks before
    // you can receive messages from xx network cMix.
    [StructLayout(LayoutKind.Sequential)]
    public class DMReceiverCallbackFunctions
    {
        DMReceiveCallbackFn receiveFn = DMReceive;
        DMReceiveTextCallbackFn receiveTextFn = DMReceiveText;
        DMReceiveReplyCallbackFn receiveReplyFn = DMReceiveReply;
        DMReceiveReactionCallbackFn receiveReactionFn = DMReceiveReaction;
        DMUpdateSentStatusCallbackFn updateSentStatusFn = DMUpdateSentStatus;
    }

    static void Main(string? ndf = null, string? stateDir = null)
    {
        if (string.IsNullOrWhiteSpace(ndf))
        {
            Console.WriteLine("Please provide an ndf json file");
            return;
        }

        if (string.IsNullOrWhiteSpace(stateDir))
        {
            Console.WriteLine("Please provide a state folder");
            return;
        }

        String ndfJSON = System.IO.File.ReadAllText(ndf);

        // CGO checks
        // Go code may not store a Go pointer in C memory. 
        // C code may store Go pointers in C memory, 
        // subject to the rule above: it must stop storing the Go pointer when 
        // the C function returns.
        Environment.SetEnvironmentVariable("GODEBUG", "cgocheck=2");

        // Setup DMReceiver Callbacks with the library.
        DMReceiverCallbackFunctions CBs = new DMReceiverCallbackFunctions();
        XXDK.cmix_dm_set_callbacks(CBs);

        Console.WriteLine(
            "#########################################" +
            "\n### .NET xxdk Shared-C Golang .dll    ###" +
            "\n#########################################\n"
        );
        GoString myver;
        myver = XXDK.GetVersion();
        Console.WriteLine($"xxdk-client version: {ConvertGoString(myver)}");

        Byte[] secret = Encoding.UTF8.GetBytes("Hello");
        Byte[] cMixParamsJSON = Encoding.UTF8.GetBytes("");
        if (!Directory.Exists(stateDir))
        {
            cMix.NewCmix(ndfJSON, stateDir, secret, "");
        }
        int cMixID = cMix.LoadCmix(stateDir, secret, cMixParamsJSON);

        Byte[] receptionID = cMix.GetReceptionID(cMixID);
        Console.WriteLine("cMix Reception ID: " +
            System.Convert.ToBase64String(receptionID));

        Byte[] dmID;
        try
        {
            dmID = cMix.EKVGet(cMixID, "MyDMID");
        }
        catch (Exception)
        {
            Console.WriteLine("Generating DM Identity...");
            dmID = cMix.GenerateCodenameIdentity("Hello");
            Console.WriteLine("Exported Codename Blob: " +
                System.Convert.ToBase64String(dmID));
            cMix.EKVSet(cMixID, "MyDMID", dmID);
        }

        Console.WriteLine("Exported Codename Blob: " +
            Encoding.UTF8.GetString(dmID));

        Int32 dmClientID = DM.NewClient(cMixID, dmID, "Hello");

        UInt32 myToken = DM.GetToken(dmClientID);
        Byte[] pubKey = DM.GetPubKey(dmClientID);
        Console.WriteLine("DMPUBKEY: " + System.Convert.ToBase64String(pubKey));
        Console.WriteLine("DMTOKEN: " + myToken);


        /*
        partnerPubKey, partnerDMToken, ok:= getDMPartner()

        if !ok {
            jww.WARN.Printf("Setting dm destination to self")

            partnerPubKey = dmID.PubKey

            partnerDMToken = dmToken

        }

        jww.INFO.Printf("DMRECVPUBKEY: %s",
            base64.RawStdEncoding.EncodeToString(partnerPubKey))

        jww.INFO.Printf("DMRECVTOKEN: %d", partnerDMToken)


        recvCh:= make(chan message.ID, 10)

        myReceiver:= &receiver{
        recv: recvCh,
			msgData: make(map[message.ID] * msgInfo),
			uuid: 0,
		}
    myNickMgr:= dm.NewNicknameManager(identity, ekv)


        sendTracker:= dm.NewSendTracker(ekv)


        dmClient:= dm.NewDMClient(&dmID, myReceiver, sendTracker,
            myNickMgr, user.GetCmix(), user.GetRng())


        err = user.StartNetworkFollower(5 * time.Second)

        if err != nil {
            jww.FATAL.Panicf("%+v", err)

        }
    // Wait until connected or crash on timeout
    connected:= make(chan bool, 10)

        user.GetCmix().AddHealthCallback(
            func(isConnected bool) {
            connected < -isConnected

            })
		waitUntilConnected(connected)

        waitForRegistration(user, 0.85)


        msgID, rnd, ephID, err:= dmClient.SendText(&partnerPubKey,
            partnerDMToken,
            viper.GetString(messageFlag),
            cmix.GetDefaultCMIXParams())

        if err != nil {
            jww.FATAL.Panicf("%+v", err)

        }
        jww.INFO.Printf("DM Send: %v, %v, %v", msgID, rnd, ephID)

        // Message Reception Loop
        waitTime:= viper.GetDuration(waitTimeoutFlag) * time.Second

        maxReceiveCnt:= viper.GetInt(receiveCountFlag)

        receiveCnt:= 0

        for done := false; !done; {
            if maxReceiveCnt != 0 && receiveCnt >= maxReceiveCnt {
                done = true

                continue

            }
            select {
			case < -time.After(waitTime):
				done = true
			case m:= < -recvCh:
				msg:= myReceiver.msgData[m]

                selfStr:= "Partner"

                if dmID.GetDMToken() == msg.dmToken {
                    selfStr = "Self"

                    if !bytes.Equal(dmID.PubKey[:],
                        msg.partnerKey[:]) {
                        jww.FATAL.Panicf(
                            "pubkey mismatch!\n")

                    }
                }
                fmt.Printf("Message received (%s, %s): %s\n",
                    selfStr, msg.mType, msg.content)

                jww.INFO.Printf("Message received: %s\n", msg)

                jww.INFO.Printf("RECVDMPUBKEY: %s",
                    base64.RawStdEncoding.EncodeToString(
                        msg.partnerKey[:]))

                jww.INFO.Printf("RECVDMTOKEN: %d", msg.dmToken)

                receiveCnt++

            }
        }
        if maxReceiveCnt == 0 {
            maxReceiveCnt = receiveCnt

        }
        fmt.Printf("Received %d/%d messages\n", receiveCnt,
            maxReceiveCnt)


        err = user.StopNetworkFollower()

        if err != nil {
            jww.WARN.Printf(
                "Failed to cleanly close threads: %+v\n",
                err)

        }
        jww.INFO.Printf("Client exiting!")
        */

        // free up allocated unmanaged memory
        /*
        if (h.IsAllocated)
        {
            h.Free();
        }
        */
    }

    private static string ConvertGoString(GoString gs)
    {
        return Marshal.PtrToStringUTF8(gs.p, unchecked((int)gs.n));
    }
    private static string ConvertCharPtr(IntPtr buf, Int32 len)
    {
        return Marshal.PtrToStringUTF8(buf, len);
    }

    static class cMix
    {
        public static void NewCmix(String ndfJSON,
            String storageDir, Byte[] password,
            string registrationCode)
        {
            GoString ndfJSONGS = NewGoString(ndfJSON);
            GoString storageDirGS = NewGoString(storageDir);
            GoSlice secret = NewGoSlice(password);
            GoString regCode = NewGoString(registrationCode);

            GoError err = XXDK.NewCmix(ndfJSONGS, storageDirGS, secret,
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

        public static int LoadCmix(String storageDir, Byte[] password,
            Byte[] cmixParamsJSON)
        {
            GoString storageDirGS = NewGoString(storageDir);
            GoSlice secret = NewGoSlice(password);
            GoSlice cmixParams = NewGoSlice(cmixParamsJSON);

            LoadCmix_return ret = XXDK.LoadCmix(storageDirGS, secret,
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

        // cmix_GetReceptionID returns the current default reception ID
        public static Byte[] GetReceptionID(Int32 cMixInstanceID)
        {
            cmix_GetReceptionID_return rid = XXDK.cmix_GetReceptionID(
                cMixInstanceID);
            GoError err = rid.Err;
            if (err.IsError != 0)
            {
                String errMsg = ConvertCharPtr(err.Msg, err.MsgLen);
                throw new Exception(errMsg);
            }
            return GoByteSliceToBytes(rid.receptionID);
        }

        public static Byte[] EKVGet(Int32 cMixInstanceID, String key)
        {
            GoString goKey = NewGoString(key);
            cmix_EKVGet_return ret = XXDK.cmix_EKVGet(cMixInstanceID, goKey);

            GoError err = ret.Err;
            if (err.IsError != 0)
            {
                String errMsg = ConvertCharPtr(err.Msg, err.MsgLen);
                throw new Exception(errMsg);
            }
            return GoByteSliceToBytes(ret.Val);
        }

        public static void EKVSet(Int32 cMixInstanceID, String key,
            Byte[] value)
        {
            GoString goKey = NewGoString(key);
            GoSlice goVal = NewGoSlice(value);
            GoError err = XXDK.cmix_EKVSet(cMixInstanceID, goKey, goVal);
            if (err.IsError != 0)
            {
                String errMsg = ConvertCharPtr(err.Msg, err.MsgLen);
                throw new Exception(errMsg);
            }

            FreeGoSlice(goVal);
            FreeGoString(goKey);
        }

        public static void StartNetworkFollower(Int32 cMixInstanceID,
            Int32 timeoutMS)
        {
            GoError err = XXDK.cmix_StartNetworkFollower(cMixInstanceID,
                timeoutMS);
            if (err.IsError != 0)
            {
                String errMsg = ConvertCharPtr(err.Msg, err.MsgLen);
                throw new Exception(errMsg);
            }
        }
        public static void StopNetworkFollower(Int32 cMixInstanceID)
        {
            GoError err = XXDK.cmix_StopNetworkFollower(cMixInstanceID);
            if (err.IsError != 0)
            {
                String errMsg = ConvertCharPtr(err.Msg, err.MsgLen);
                throw new Exception(errMsg);
            }
        }
        public static void WaitForNetwork(Int32 cMixInstanceID,
            Int32 timeoutMS)
        {
            GoError err = XXDK.cmix_WaitForNetwork(cMixInstanceID, timeoutMS);
            if (err.IsError != 0)
            {
                String errMsg = ConvertCharPtr(err.Msg, err.MsgLen);
                throw new Exception(errMsg);
            }
        }
        public static Boolean ReadyToSend(Int32 cMixInstanceID)
        {
            GoUint8 ready = XXDK.cmix_ReadyToSend(cMixInstanceID);
            if (ready != 0) {
                return true;
            } 
            return false;
        }

        public static Byte[] GenerateCodenameIdentity(
            String secretPassphrase)
        {
            GoString secret = NewGoString(secretPassphrase);
            Console.WriteLine("Secret: " + secretPassphrase);
            GoByteSlice id = XXDK.cmix_GenerateCodenameIdentity(secret);
            FreeGoString(secret);
            return GoByteSliceToBytes(id);
        }

    }

    static class DM
    {
        public static Int32 NewClient(Int32 cMixInstanceID,
            Byte[] codenameIdentity, String secretPassphrase)
        {
            GoSlice id = NewGoSlice(codenameIdentity);
            Console.WriteLine("SecretNew: " + secretPassphrase);
            GoString secret = NewGoString(secretPassphrase);
            cmix_dm_NewDMClient_return ret = XXDK.cmix_dm_NewDMClient(
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
        public static UInt32 GetToken(Int32 dmInstanceID)
        {
            cmix_dm_GetDMToken_return ret = XXDK.cmix_dm_GetDMToken(
                dmInstanceID);
            GoError err = ret.Err;
            if (err.IsError != 0)
            {
                String errMsg = ConvertCharPtr(err.Msg, err.MsgLen);
                throw new Exception(errMsg);
            }
            return ret.Token;
        }
        public static Byte[] GetPubKey(Int32 dmInstanceID)
        {
            cmix_dm_GetDMPubKey_return ret = XXDK.cmix_dm_GetDMPubKey(
                dmInstanceID);
            GoError err = ret.Err;
            if (err.IsError != 0)
            {
                String errMsg = ConvertCharPtr(err.Msg, err.MsgLen);
                throw new Exception(errMsg);
            }
            return GoByteSliceToBytes(ret.PubKey);
        }
        public static Byte[] SendText(Int32 dmInstanceID,
            Byte[] partnerPubKey, UInt32 dmToken,
            String message, Int64 leaseTimeMS, Byte[] cmixParamsJSON)
        {
            GoSlice partnerKey = NewGoSlice(partnerPubKey);
            GoString goMsg = NewGoString(message);
            GoSlice cmixParams = NewGoSlice(cmixParamsJSON);
            cmix_dm_SendText_return ret = XXDK.cmix_dm_SendText(dmInstanceID,
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


    static class XXDK
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