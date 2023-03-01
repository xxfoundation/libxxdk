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
    public GoSlice r0;
    public GoError r1;
}
/* Return type for cmix_EKVGet */
[StructLayout(LayoutKind.Sequential)]
struct cmix_EKVGet_return
{
    public GoSlice r0;
    public GoError r1;
}
/* Return type for cmix_dm_NewDMClient */
[StructLayout(LayoutKind.Sequential)]
struct cmix_dm_NewDMClient_return
{
    public GoInt r0;
    public GoError r1;
}
/* Return type for cmix_dm_GetDMToken */
[StructLayout(LayoutKind.Sequential)]
struct cmix_dm_GetDMToken_return
{
    public GoUint32 r0;
    public GoError r1;
}
/* Return type for cmix_dm_GetDMPubKey */
[StructLayout(LayoutKind.Sequential)]
struct cmix_dm_GetDMPubKey_return
{
    public GoSlice r0;
    public GoError r1;
}
/* Return type for cmix_dm_SendText */
[StructLayout(LayoutKind.Sequential)]
struct cmix_dm_SendText_return
{
    public GoSlice r0;
    public GoError r1;
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
        return gs;
    }
    private static void FreeGoSlice(GoSlice freeMe)
    {
        Marshal.FreeHGlobal(freeMe.data);
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
        /*



        // call the external functions

        int addResult = GoMath.Add(a, b);
        int subResult = GoMath.Sub(a, b);
        double cosineResult = GoMath.Cosine(x);
        int sumResult = GoMath.Sum(gs);
        var helloResult = GoHello.HelloWorld(s);
        GoMath.Sort(gs);

        // Read the Sorted GoSlice

        n = (int)gs.len;
        Int64[] arr = new Int64[n];

        for (int i = 0; i < n; i++)
        {
            arr[i] = Marshal.ReadInt64(gs.data, i * Marshal.SizeOf(typeof(Int64)));
        }

        // Read the size of the data returned by HelloWorld
        // The size is an int32, so we read 4 bytes

        byte* buf = (byte*)helloResult;
        byte[] lenBytes = new byte[4];

        for (int i = 0; i < 4; i++)
        {
            lenBytes[i] = *buf++;
        }

        // Read the result itself

        n = BitConverter.ToInt32(lenBytes, 0);
        int j = 0;
        byte[] bytes = new byte[n];

        for (int i = 0; i < n; i++)
        {
            // Skip the first 4 bytes because
            // they hold the size

            if (i < 4)
            {
                *buf++ = 0;
            }
            else
            {
                bytes[j] = *buf++;
                j++;
            }
        }

        // Print results
        */
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
        /*
        cmixParams, _:= initParams()

        user:= loadOrInitCmix([]byte(viper.GetString(passwordFlag)),
			viper.GetString(sessionFlag), "", cmixParams)

		// Print user's reception ID
		identity:= user.GetStorage().GetReceptionID()

        jww.INFO.Printf("User: %s", identity)

        // NOTE: DM ID's are not storage backed, so we do the
        // storage here.
        ekv:= user.GetStorage().GetKV()

        rng:= user.GetRng().GetStream()

        defer rng.Close()

        dmIDObj, err:= ekv.Get("dmID", 0)

        if err != nil && ekv.Exists(err) {
            jww.FATAL.Panicf("%+v", err)

        }
        var dmID codename.PrivateIdentity

        if ekv.Exists(err) {
            dmID, err = codename.UnmarshalPrivateIdentity(
                dmIDObj.Data)

        }
        else
        {
            dmID, err = codename.GenerateIdentity(rng)

        }
        if err != nil {
            jww.FATAL.Panicf("%+v", err)

        }
    dmToken:= dmID.GetDMToken()

        pubKeyBytes:= dmID.PubKey[:]


        ekv.Set("dmID", &versioned.Object{
        Version: 0,
			Timestamp: time.Now(),
			Data: dmID.Marshal(),
		})

		jww.INFO.Printf("DMPUBKEY: %s",
            base64.RawStdEncoding.EncodeToString(pubKeyBytes))

        jww.INFO.Printf("DMTOKEN: %d", dmToken)


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
        return Marshal.PtrToStringAnsi(gs.p, unchecked((int)gs.n));
    }
    private static string ConvertCharPtr(IntPtr buf, Int32 len)
    {
        return Marshal.PtrToStringAnsi(buf, len);
    }


    // Prints an Int64 array to a pretty string
    private static string Int64ArrayToString(Int64[] arr)
    {
        var strBuilder = new StringBuilder("");
        var n = arr.Length;

        for (int i = 0; i < n; i++)
        {
            if (i == (n - 1))
            {
                strBuilder = strBuilder.Append($"{arr[i]}\n");
            }

            else if (i == 0)
            {
                strBuilder = strBuilder.Append($"Sort: {arr[i]}, ");
            }

            else
            {
                strBuilder = strBuilder.Append($"{arr[i]}, ");
            }
        }

        return strBuilder.ToString();
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
        public static extern cmix_GetReceptionID_return cmix_GetReceptionID(GoInt32 cMixInstanceID);
        [DllImport("libxxdk.so")]
        public static extern cmix_EKVGet_return cmix_EKVGet(GoInt32 cMixInstanceID, GoString key);
        [DllImport("libxxdk.so")]
        public static extern GoError cmix_EKVSet(GoInt32 cMixInstanceID, GoString key, GoSlice value);
        [DllImport("libxxdk.so")]
        public static extern GoError cmix_StartNetworkFollower(GoInt cMixInstanceID, GoInt timeoutMS);
        [DllImport("libxxdk.so")]
        public static extern GoError cmix_StopNetworkFollower(GoInt cMixInstanceID);
        [DllImport("libxxdk.so")]
        public static extern GoError cmix_WaitForNetwork(GoInt cMixInstanceID, GoInt timeoutMS);
        [DllImport("libxxdk.so")]
        public static extern GoUint8 cmix_ReadyToSend(GoInt cMixInstanceID);
        [DllImport("libxxdk.so")]
        public static extern GoSlice cmix_GenerateCodenameIdentity(GoString secretPassphrase);
        [DllImport("libxxdk.so")]
        public static extern cmix_dm_NewDMClient_return cmix_dm_NewDMClient(GoInt cMixInstanceID, GoSlice codenameIdentity, GoString secretPassphrase);
        [DllImport("libxxdk.so")]
        public static extern cmix_dm_GetDMToken_return cmix_dm_GetDMToken(GoInt dmInstanceID);
        [DllImport("libxxdk.so")]
        public static extern cmix_dm_GetDMPubKey_return cmix_dm_GetDMPubKey(GoInt dmInstanceID);
        [DllImport("libxxdk.so")]
        public static extern cmix_dm_SendText_return cmix_dm_SendText(GoInt dmInstanceID, GoSlice partnerPubKey, GoUint32 dmToken, GoString message, GoInt64 leaseTimeMS, GoSlice cmixParamsJSON);

    }

}