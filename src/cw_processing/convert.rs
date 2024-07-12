use std::{
    ffi::{CStr, CString},
    path::PathBuf,
    ptr::null,
    time::Instant,
};

use libc::c_void;
use tokio::fs;

use crate::{args::ARGS, cw_processing::ffi::cw_import_xml, libc_freer::LibcFreer};

pub async fn xml(file_path: PathBuf) {
    unsafe {
        let now = Instant::now();
        println!(
            "processing file: {:?}",
            file_path.file_name().expect("no file name")
        );

        let res = tokio::task::spawn_blocking(move || {
            let path = file_path
                .to_str()
                .expect("cant convert path to string")
                .to_owned();

            let c_path = CString::new(path).expect("cstring creation failed");

            return cw_import_xml(c_path.as_ptr() as *const u8);
        })
        .await
        .expect("spawing processing task failed");

        let mut out_path = PathBuf::new();

        if res.data == null() || res.file_name == null() {
            println!("null ptr from cw");
            return;
        }

        let slice = std::slice::from_raw_parts(res.data, res.data_len);
        let data = LibcFreer::new(slice, res.data as *mut c_void);

        if let Ok(file_name) = CStr::from_ptr(res.file_name as *const i8).to_str() {
            let file_name = file_name.to_owned();
            libc::free(res.file_name as *mut libc::c_void);

            println!("import xml {} took: {:.2?}", file_name, now.elapsed());

            let out_dir = ARGS
                .get()
                .expect("no args")
                .output_dir
                .to_str()
                .expect("cant convert path to string");

            out_path.push(out_dir);

            out_path.push(file_name);

            if !fs::try_exists(&out_dir)
                .await
                .expect("unable to check if dir exsists")
            {
                println!("creating output dir");
                fs::create_dir_all(out_dir)
                    .await
                    .expect("unable to create output directory");
            }

            fs::write(out_path, *data)
                .await
                .unwrap_or_else(|err| println!("error while writing res file {:?}", err));
        }
    }
}
