using CodeWalker.GameFiles;
using System;
using System.IO;
using System.Runtime.InteropServices;
using System.Xml;
using static CWlib.CWlib;

namespace CWlib
{
    internal class CWlib
    {
        [UnmanagedCallersOnly(EntryPoint = "add_dotnet")]
        public static int Add(int a, int b)
        {
            return a + b;
        }

        [UnmanagedCallersOnly(EntryPoint = "test_type")]
        public static nint TestType(nint a)
        {
            var trim = 4;
            var payload = Marshal.PtrToStringUTF8(a);
            payload = payload.Substring(0, payload.Length - 1);
            Console.WriteLine(payload);
            return Marshal.StringToHGlobalAnsi(XmlMeta.GetXMLFormat(payload, out trim).ToString());
        }

        [StructLayout(LayoutKind.Sequential)]
        public struct CwCallRes
        {
            public nint data;
            public nuint data_len;
            public nint file_name;

        }

        [UnmanagedCallersOnly(EntryPoint = "cw_import_xml")]
        public static CwCallRes CwImportXml(nint raw_path)
        {
            var path = Marshal.PtrToStringUTF8(raw_path);

            if (path.Last() == '\n')
            {
                path = path.Remove(path.Length - 1);
            }

            Console.WriteLine(path);

            if (!File.Exists(path))
            {
                Console.WriteLine("cw: no file");
                return new CwCallRes { };
            }

            var fi = new FileInfo(path);
            var fname = fi.Name;
            var fnamel = fname.ToLowerInvariant();
            var fpathin = path;

            var trimlength = 4;
            var mformat = XmlMeta.GetXMLFormat(fnamel, out trimlength);
            Console.WriteLine(mformat);

            fname = fname.Substring(0, fname.Length - trimlength);
            fnamel = fnamel.Substring(0, fnamel.Length - trimlength);
            fpathin = fpathin.Substring(0, fpathin.Length - trimlength);
            fpathin = Path.Combine(Path.GetDirectoryName(fpathin), Path.GetFileNameWithoutExtension(fpathin));

            var doc = new XmlDocument();
            string text = File.ReadAllText(path);
            if (!string.IsNullOrEmpty(text))
            {
                doc.Load(path);
            }

            byte[] data = XmlMeta.GetData(doc, mformat, fpathin);

            if (data != null)
            {

                var ptr = Marshal.AllocHGlobal(data.Length * sizeof(byte));
                Marshal.Copy(data, 0, ptr, data.Length);

                var len = data.Length;
                data = null;

                return new CwCallRes { data = ptr, data_len = (nuint)len, file_name = Marshal.StringToHGlobalAnsi(fname) };
            }

            return new CwCallRes { };


            /* unsafe
             {
                 byte[] test = {1, 2, 3, 4,2, 23, 32, 12, 213, 123, 43, 133, 45, 86, 234, 99, 76, 19};
                 var unm = Marshal.AllocHGlobal(sizeof(byte) * 4);
                 Marshal.Copy(test, 0, unm, test.Length);
                 return new CwCallRes { data_len = (nuint)test.Length, data = unm, file_name = Marshal.StringToHGlobalAnsi("123") };
             }*/
        }

        [UnmanagedCallersOnly(EntryPoint = "gc_collect")]
        public static void GcCollect(nint ptr)
        {
            GC.Collect();
        }
    }
}
