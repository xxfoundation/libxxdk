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
using DM = XX.Network.DirectMessaging;

/// <summary>
/// Main Program
/// </summary>
public class Program
{

    static void Main(string? ndf = null, string? stateDir = null,
        string? partnerKey = null, uint partnerToken= 0, string? message = null)
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
        XX.Network.SetupReceiverCallbacks();

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
            dmID = XX.Network.GenerateCodenameIdentity("Hello");
            Console.WriteLine("Exported Codename Blob: " +
                System.Convert.ToBase64String(dmID));
            cMix.EKVSet(cMixID, "MyDMID", dmID);
        }

        Console.WriteLine("Exported Codename Blob: " +
            Encoding.UTF8.GetString(dmID));

        Int32 dmClientID = DM.NewClient(cMixID,
            dmID, "Hello");

        UInt32 myToken = DM.GetToken(dmClientID);
        Byte[] pubKey = DM.GetPubKey(dmClientID);
        Console.WriteLine("DMPUBKEY: " + System.Convert.ToBase64String(pubKey));
        Console.WriteLine("DMTOKEN: " + myToken);

        UInt32 partnerDMToken;
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

        cMix.StartNetworkFollower(cMixID, 5000);

        Boolean connected = false;
        while (!connected)
        {
            try
            {
                cMix.WaitForNetwork(cMixID, 20000);
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
                registered = cMix.ReadyToSend(cMixID);
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

        DM.SendText(dmClientID, partnerKeyBytes, partnerDMToken, message, 0,
            cMixParamsJSON);

        // Sleep for 20s
        // TODO: Normally we'd wait until we receive messages
        System.Threading.Thread.Sleep(20000);


        cMix.StopNetworkFollower(cMixID);

    }

}