use std::{env, path::PathBuf, process::Command};

use ignore::{types::TypesBuilder, WalkBuilder};

fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let csharp_project = "CWlib";

    let (rid, pattern) = if cfg!(windows) {
        ("win-x64", "sdk\\System.Private.CoreLib.dll")
    } else {
        ("linux-x64", "sdk/System.Private.CoreLib.dll")
    };

    let output = Command::new("dotnet")
        .args(["publish", "-v", "d", "-r", rid, "-c", "Release"])
        .current_dir(PathBuf::from(manifest_dir).join(csharp_project))
        .output()
        .unwrap();
    let out: String = String::from_utf8_lossy(&output.stdout).into();
    if !output.status.success() {
        let err: String = String::from_utf8_lossy(&output.stderr).into();
        panic!("Error: \n{}\n{}\n", out, err);
    }

    let core_lib_path: PathBuf = out
        .find(pattern)
        .and_then(|pos| {
            let matched = &out[..pos + pattern.len()];
            if cfg!(windows) {
                matched.rfind(':').map(|begin| &matched[begin - 1..])
            } else {
                matched.rfind("/home").map(|begin| &matched[begin..])
            }
        })
        .unwrap_or_else(|| {
            std::fs::write("dotnet-output.txt", &out).unwrap();
            panic!("ILCompiler sdk path not found in the dotnet command output: dotnet-output.txt")
        })
        .into();
    let ilcompiler_path = core_lib_path
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_str()
        .unwrap();

    let mut types = TypesBuilder::new();
    types.add("cs", "*.cs").unwrap();
    types.add("csproj", "*.csproj").unwrap();
    types.select("cs").select("csproj");
    let walker = WalkBuilder::new(csharp_project)
        .types(types.build().unwrap())
        .build();
    for file in walker {
        let file = file.unwrap();
        if file.file_type().map(|t| t.is_file()).unwrap_or_default() {
            println!("cargo:rerun-if-changed={}", file.path().display());
        }
    }

    println!("cargo:rustc-link-search={ilcompiler_path}/sdk");
    println!("cargo:rustc-link-search={ilcompiler_path}/framework");

    let vcpkg = env::var("VCPKG_ROOT");
    if let Ok(path) = vcpkg {
        println!("cargo:rustc-link-search={path}/installed/x64-windows/lib");
    };

    println!("cargo:rustc-link-lib=static={csharp_project}");
    println!(
        "cargo:rustc-link-search={manifest_dir}/{csharp_project}/bin/Release/net7.0/{rid}/publish"
    );

    if cfg!(windows) {
        println!("cargo:rustc-link-arg=/INCLUDE:NativeAOT_StaticInitialization");
        println!("cargo:rustc-link-lib=static=System.Globalization.Native.Aot");
        println!("cargo:rustc-link-lib=static=System.IO.Compression.Native.Aot");
    } else {
        println!("cargo:rustc-link-arg=-Wl,--require-defined,NativeAOT_StaticInitialization");
        println!("cargo:rustc-link-arg=-lstdc++");
        println!("cargo:rustc-link-lib=static=System.Globalization.Native");
        println!("cargo:rustc-link-lib=static=System.Native");
    }
    println!("cargo:rustc-link-lib=static=bootstrapperdll");
    println!("cargo:rustc-link-lib=static=Runtime.WorkstationGC");
    println!("cargo:rustc-link-lib=static=brotlicommon");
    println!("cargo:rustc-link-lib=static=brotlienc");
    println!("cargo:rustc-link-lib=static=brotlidec");
}
