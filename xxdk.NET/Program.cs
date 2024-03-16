using System;
using System.Buffers.Text;
using System.Diagnostics.Metrics;
using System.Net.Sockets;
using System.Runtime.InteropServices;
using System.Runtime.Intrinsics.X86;
using System.Security.Principal;
using System.Text;
using System.CommandLine;

using XX;

namespace Program;

using cMix = XX.Network.CMix;
using DirectMessaging = XX.Network.DirectMessaging;

/// <summary>
/// Main Program
/// </summary>
public class Program
{

    static void Main(string? ndf = null, string? stateDir = null,
        string? partnerKey = null, int partnerToken= 0, string? message = null,
        int wait = 20, Int64 receiveCount = 1)
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

        // TODO: this should be in the library
        Environment.SetEnvironmentVariable("GODEBUG", "cgocheck=2");
        // Setup DMReceiver Callbacks with the library.
        // TODO: The callbacks should be settable by the user, for now
        //       all we're doing is printing the messsages to console...
        XX.Network.SetupReceiverRouter();

        Console.WriteLine(
            "#########################################" +
            "\n### .NET xxdk example demo            ###" +
            "\n#########################################\n"
        );
        Console.WriteLine($"xxdk-client version: {XX.Network.GetVersion()}");

        Byte[] secret = Encoding.UTF8.GetBytes("Hello");
        Byte[] cMixParamsJSON = Encoding.UTF8.GetBytes("");
        if (!Directory.Exists(stateDir))
        {
            cMix.NewCmix(ndfJSON, stateDir, secret, "");
        }
        // TODO: Instead of tracking these IDs c-lib style, we should
        //       return class intances that do it for us.
        cMix net = cMix.LoadCmix(stateDir, secret, cMixParamsJSON);

        Byte[] receptionID = net.GetReceptionID();
        Console.WriteLine("cMix Reception ID: " +
            System.Convert.ToBase64String(receptionID));

        Byte[] dmID;
        try
        {
            dmID = net.EKVGet("MyDMID");
        }
        catch (Exception)
        {
            Console.WriteLine("Generating DM Identity...");
            dmID = XX.Network.GenerateCodenameIdentity("Hello");
            Console.WriteLine("Exported Codename Blob: " +
                System.Convert.ToBase64String(dmID));
            net.EKVSet("MyDMID", dmID);
        }

        Console.WriteLine("Exported Codename Blob: " +
            Encoding.UTF8.GetString(dmID));

        DMCallbacks dmCbs = new();

        DirectMessaging DM = new(net, dmID, "Hello", dmCbs);

        Int32 myToken = DM.GetToken();
        Byte[] pubKey = DM.GetPubKey();
        Console.WriteLine("DMPUBKEY: " + System.Convert.ToBase64String(pubKey));
        Console.WriteLine("DMTOKEN: " + myToken);

        Int32 partnerDMToken;
        Byte[] partnerKeyBytes;
        if (string.IsNullOrWhiteSpace(partnerKey) || partnerToken == 0)
        {
            Console.WriteLine("Partner key or token missing, sending to self");
            partnerDMToken = myToken;
            partnerKeyBytes = pubKey;
        }
        else
        {
            partnerDMToken = partnerToken;
            partnerKeyBytes = System.Convert.FromBase64String(partnerKey);
        }
        Console.WriteLine("PARTNERDMPUBKEY: " +
            System.Convert.ToBase64String(partnerKeyBytes));
        Console.WriteLine("PARTNERDMTOKEN: " + partnerDMToken);

        net.StartNetworkFollower(5000);

        Boolean connected = false;
        while (!connected)
        {
            try
            {
                net.WaitForNetwork(20000);
                connected = true;
            }
            catch (Exception e)
            {
                Console.WriteLine("Waiting to connect: " + e);
            }
        }
        // We won't use the no-reg version for this, so lets wait until
        // we are registered enough to send.
        Boolean registered = false;
        while (!registered)
        {
            try
            {
                registered = net.ReadyToSend();
                System.Threading.Thread.Sleep(1000);
            }
            catch (Exception e)
            {
                Console.WriteLine("Exception waiting to register: " + e);
            }
        }

        if (string.IsNullOrWhiteSpace(message))
        {
            message = "Hello, World!";
        }

        DM.SendText(partnerKeyBytes, partnerDMToken, message, 0,
            cMixParamsJSON);

        // wait at most wait times
        int waitCnt = 0;
        while (dmCbs.GetNumReceived() < receiveCount && waitCnt < wait)
        {
            Console.WriteLine("Num Received: {0}...", dmCbs.GetNumReceived());
            System.Threading.Thread.Sleep(1000); // 1s per wait
            waitCnt++;
        }

        if (waitCnt >= wait)
        {
            Console.WriteLine("Timed out waiting for messages!");
        }

        Console.WriteLine("Num Received: {0}...Exiting!",
            dmCbs.GetNumReceived());

        net.StopNetworkFollower();


    }
    private class DMCallbacks : XX.Network.IDMReceiver
    {
        private Int64 numReceived;
        public DMCallbacks()
        {
            this.numReceived = 0;
        }

        public Int64 GetNumReceived()
        {
            return this.numReceived;
        }

        /// <summary>
        /// Receive RAW direct message callback
        /// </summary>
        public Int64 Receive(Byte[] message_id, String nickname,
            Byte[] text, Byte[] partnerkey, Byte[] senderkey, Int32 dmToken,
            Int32 codeset, Int64 timestamp, Int64 round_id, Int64 msg_type,
            Int64 status)
        {
            Console.WriteLine("DMReceiveCallbackFn");
            this.numReceived++;
            return this.numReceived;
        }

        /// <summary>
        /// Received Text message callback
        /// </summary>
        public Int64 ReceiveText(Byte[] message_id, String nickname,
            String text, Byte[] partnerkey, Byte[] senderkey, Int32 dmToken,
            Int32 codeset, Int64 timestamp, Int64 round_id, Int64 status)
        {
            Console.WriteLine("DMReceiveTextCallbackFn");
            this.numReceived++;
            return this.numReceived;

        }

        /// <summary>
        /// Received Reply message callback
        /// </summary>
        public Int64 ReceiveReply(Byte[] message_id, Byte[] reply_to,
            String nickname, String text, Byte[] partnerkey, Byte[] senderkey,
            Int32 dmToken, Int32 codeset, Int64 timestamp, Int64 round_id,
            Int64 status)
        {
            Console.WriteLine("DMReceiveReplyCallbackFn");
            this.numReceived++;
            return this.numReceived;

        }

        /// <summary>
        /// Received Reaction message callback
        /// </summary>
        public Int64 ReceiveReaction(Byte[] message_id, Byte[] reaction_to,
            String nickname, String text, Byte[] partnerkey, Byte[] senderkey,
            Int32 dmToken, Int32 codeset, Int64 timestamp, Int64 round_id,
            Int64 status)
        {
            Console.WriteLine("DMReceiveReactionCallbackFn");
            this.numReceived++;
            return this.numReceived;
        }

        /// <summary>
        /// Message was updated callback. Used to tell UI progress as
        /// message is sent through the network. 
        /// </summary>
        public Int64 UpdateSentStatus(Int64 uuid, Byte[] message_id,
            Int64 timestamp, Int64 round_id, Int64 status)
        {
            Console.WriteLine("DMUpdateSentStatusCallbackFn");
            this.numReceived++;
            return this.numReceived;
        }

        public void BlockUser(byte[] pubkey)
        {
            return;
        }

        public void UnblockUser(byte[] pubkey)
        {
            return;
        }

        public byte[] GetConversation(byte[] senderkey)
        {
            Byte[] conv = Array.Empty<byte>();
            return conv;
        }

        public byte[] GetConversations()
        {
            Byte[] conv = Array.Empty<byte>();
            return conv;
        }

        public bool DeleteMessage(byte[] messageID, byte[] pubkey)
        {
            return false;
        }

        public void EventUpdate(Int64 eventType, byte[] jsonData)
        {
            Console.WriteLine("Event Update: " + jsonData);
        }
    }
}
