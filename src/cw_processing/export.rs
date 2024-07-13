use std::{ffi::CString, path::PathBuf, time::Instant};

use crate::cw_processing::ffi;

pub async fn texture_dict(file_path: PathBuf) {
    unsafe {
        let now = Instant::now();
        println!(
            "processing file: {:?}",
            file_path.file_name().expect("no file name")
        );

        let path = file_path.clone();

        let file_name = file_path
            .file_name()
            .expect("invalid filename")
            .to_str()
            .expect("invalid filename");

        tokio::task::spawn_blocking(move || {
            let path = path
                .to_str()
                .expect("cant convert path to string")
                .to_owned();

            let c_path = CString::new(path).expect("cstring creation failed");

            return ffi::cw_export_texture_dict(c_path.as_ptr() as *const u8);
        })
        .await
        .expect("spawing processing task failed");

        println!("ytd export {} took: {:.2?}", file_name, now.elapsed());
    }
}
