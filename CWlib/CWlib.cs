using CodeWalker.GameFiles;
using System.Runtime.InteropServices;
using System.Xml;

namespace CWlib
{
    internal class CWlib
    {
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

            if (!File.Exists(path))
            {
                Console.WriteLine("cw: no file");
                return new CwCallRes { };
            }

            var fi = new FileInfo(path);
            var fname = fi.Name;
            var fnamel = fname.ToLowerInvariant();
            var fpathin = path;

            var mformat = XmlMeta.GetXMLFormat(fnamel, out int trimlength);
            Console.WriteLine(mformat);

            fname = fname.Substring(0, fname.Length - trimlength);
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

                return new CwCallRes { data = ptr, data_len = (nuint)len, file_name = Marshal.StringToHGlobalAnsi(fname) };
            }

            return new CwCallRes { };
        }

        [UnmanagedCallersOnly(EntryPoint = "gc_collect")]
        public static void GcCollect(nint ptr)
        {
            GC.Collect();
        }
    }
}