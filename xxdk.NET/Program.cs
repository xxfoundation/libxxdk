using System;
using System.Diagnostics.Metrics;
using System.Runtime.InteropServices;
using System.Text;

namespace SharedC
{
    unsafe class Program
    {
        /// <summary>
        /// Go Data Structures
        /// </summary>

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

        /// <summary>
        /// Callback Functions to implement.
        /// </summary>
        private delegate long DMReceiveCallbackFn(int dm_instance_id,
            void* message_id, int message_id_len,
            char* nickname, int nickname_len,
            void* text, int text_len,
            void* pubkey, int pubkey_len,
            int dmToken, int codeset,
            long timestamp, long round_id, long msg_type, long status);
        private delegate long DMReceiveTextCallbackFn(int dm_instance_id,
            void* mesage_id, int message_id_len,
            char* nickname, int nickname_len,
            char* text, int text_len,
            void* pubkey, int pubkey_len,
            int dmToken, int codeset,
            long timestamp, long round_id, long status);
        private delegate long DMReceiveReplyCallbackFn(int dm_instance_id,
            void* mesage_id, int message_id_len,
            void* reply_to, int reply_to_len,
            char* nickname, int nickname_len,
            char* text, int text_len,
            void* pubkey, int pubkey_len,
            int dmToken, int codeset,
            long timestamp, long round_id, long status);
        private delegate long DMReceiveReactionCallbackFn(int dm_instance_id,
            void* mesage_id, int message_id_len,
            void* reaction_to, int reaction_to_len,
            char* nickname, int nickname_len,
            char* text, int text_len,
            void* pubkey, int pubkey_len,
            int dmToken, int codeset,
            long timestamp, long round_id, long status);
        private delegate long DMUpdateSentStatusCallbackFn(int dm_instance_id,
            long uuid,
            void* message_id, int message_id_len, long timestamp,
            long round_id, long status);


        static void Main(string[] args)
        {
            // CGO checks
            // Go code may not store a Go pointer in C memory. 
            // C code may store Go pointers in C memory, 
            // subject to the rule above: it must stop storing the Go pointer when 
            // the C function returns.

            Environment.SetEnvironmentVariable("GODEBUG", "cgocheck=2");

            // define parameters
            /*
            int a = 10;
            int b = 2;
            double x = 100;
            Int64[] t = new Int64[] { 35, 56, 1, 3, 2, 88, 14 };

            // Allocate unmanaged memory for
            // the GoSlice

            int n = t.Length;
            GCHandle h = GCHandle.Alloc(t, GCHandleType.Pinned);
            GoSlice gs = new GoSlice
            {
                data = h.AddrOfPinnedObject(),
                cap = n,
                len = n
            };

            // Allocate unmanaged memory for the
            // GoString

            string msg = "I am the Hal 9000";
            GoString s = new GoString
            {
                p = Marshal.StringToHGlobalAnsi(msg),
                n = msg.Length
            };

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
                "\n### .NET Calling Shared-C Golang .dll ###" +
                "\n#########################################\n"
            );
            GoString myver;
            myver = XXDK.GetVersion();
            Console.WriteLine($"HelloWorld: {ConvertGoString(myver)}");
            myver = XXDK.GetDependencies();
            Console.WriteLine($"HelloWorld: {ConvertGoString(myver)}");

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

        [StructLayout(LayoutKind.Sequential)]
        public class DMReceiverCallbackFunctions
        {
            DMReceiveCallbackFn? receiveFn;
            DMReceiveTextCallbackFn? receiveTextFn;
            DMReceiveReplyCallbackFn? receiveReplyFn;
            DMReceiveTextCallbackFn? receiveReactionFn;
            DMUpdateSentStatusCallbackFn? updateSentStatusFn;
        }
        static class XXDK
        {
            [DllImport("libxxdk.so")]
            public static extern void cmix_dm_set_callbacks(DMReceiverCallbackFunctions cbs);

            [DllImport("libxxdk.so")]
            public static extern GoString GetVersion();
            [DllImport("libxxdk.so")]
            public static extern GoString GetDependencies();
        }

    }
}