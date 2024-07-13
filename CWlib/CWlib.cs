using CodeWalker.GameFiles;
using CodeWalker.Utils;
using CodeWalker.World;
using System.Runtime.InteropServices;
using System.Xml;
using static CodeWalker.Utils.DDSIO;

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

        [UnmanagedCallersOnly(EntryPoint = "cw_export_texture_dict")]
        public static void CwExportTextureDict(nint raw_path)
        {
            var path = Marshal.PtrToStringUTF8(raw_path);

            if (path.Last() == '\n')
            {
                path = path.Remove(path.Length - 1);
            }

            if (!File.Exists(path))
            {
                Console.WriteLine("cw: no file");
                return;
            }

            byte[] data = File.ReadAllBytes(path);

            YtdFile ytd = new YtdFile();
            ytd.Load(data);

            if (ytd.TextureDict.Textures?.data_items == null) return;

            var outDir = Path.Combine(Path.GetDirectoryName(path), Path.GetFileNameWithoutExtension(path));
            
            if (!Directory.Exists(outDir))
            {
                Directory.CreateDirectory(outDir);
            }

            foreach (var tex in ytd.TextureDict.Textures.data_items)
            {
                try {
                    byte[] dds = DDSIO.GetDDSFile(tex);
                    string bpath = outDir + "\\" + tex.Name;
                    string fpath = bpath + ".dds";
                    int c = 1;
                    while (File.Exists(fpath))
                    {
                        fpath = bpath + "_Copy" + c.ToString() + ".dds";
                        c++;
                    }
                File.WriteAllBytes(fpath, dds);
                } catch (Exception e) {
                    Console.WriteLine("error while processing {0} :: {1} - {2}", Path.GetFileName(path), tex.Name, e);
                }


            }
        }

        [UnmanagedCallersOnly(EntryPoint = "gc_collect")]
        public static void GcCollect(nint ptr)
        {
            GC.Collect();
        }
    }
}