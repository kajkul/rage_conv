mod libc_freer;

use core::{ffi, time};
use std::{
    collections::HashMap,
    ffi::{CStr, CString},
    os::raw::c_void,
    path::{Path, PathBuf},
    ptr::null,
    sync::{self, Arc, Mutex},
    time::{Duration, Instant},
};

use libc_freer::LibcFreer;
use notify::{EventKind, RecursiveMode, Watcher};
use tokio::{fs, select};

extern crate windows;

#[repr(C)]
pub struct RawCwCallRes {
    data: *const u8,
    data_len: usize,
    file_name: *const u8,
}

extern "C" {
    pub fn cw_import_xml(path: *const u8) -> RawCwCallRes;
    pub fn gc_collect();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = sync::mpsc::channel::<PathBuf>();
    let map = Arc::new(Mutex::new(HashMap::<
        PathBuf,
        tokio::sync::oneshot::Sender<()>,
    >::new()));

    let mut watcher =
        notify::recommended_watcher(move |res: notify::Result<notify::Event>| match res {
            Ok(event) => match event.kind {
                EventKind::Create(_) | EventKind::Modify(_) => {
                    event.paths.into_iter().for_each(|path| {
                        if let Some(ending) = path.extension() {
                            if let Some(ext) = ending.to_str() {
                                if ext == "xml" {
                                    tx.send(path).unwrap();
                                }
                            }
                        }
                    })
                }
                _ => (),
            },
            Err(e) => println!("watch error: {:?}", e),
        })?;

    watcher.watch(Path::new("./test"), RecursiveMode::Recursive)?;

    // Kind of a hacky way to force C# runtime to do gc
    tokio::spawn(async {
        loop {
            tokio::time::sleep(time::Duration::from_secs(30)).await;
            unsafe { gc_collect() };
        }
    });

    for path in rx {
        let map = map.clone();
        tokio::spawn(async move {
            let (tx, rx) = tokio::sync::oneshot::channel::<()>();

            {
                let mut guard = map.lock().unwrap();
                if let Some(tx) = guard.remove(&path) {
                    tx.send(())
                        .unwrap_or_else(|err| println!("error while waiting for file: {:?}", err));
                }

                guard.insert(path.clone(), tx);
            }

            select! {
              _ = rx => {
                return
              }
              _ = tokio::time::sleep(Duration::from_millis(250)) => {
                if let Some(path) = remove_from_map(map, &path) {
                  process_file(path).await
                }
              }
            }
        });
    }

    Ok(())
}

fn remove_from_map(
    map: Arc<Mutex<HashMap<PathBuf, tokio::sync::oneshot::Sender<()>>>>,
    path: &PathBuf,
) -> Option<PathBuf> {
    let mut guard = map.lock().unwrap();
    if let Some((key, _)) = guard.remove_entry(path) {
        return Some(key);
    }
    None
}

async fn process_file(path: PathBuf) {
    match tokio::task::spawn_blocking(move || import_xml(path)).await {
        Ok(res) => {
            match res {
                Some((data, path)) => {
                    fs::write(path, *data)
                        .await
                        .unwrap_or_else(|err| println!("error while writing res file {:?}", err));
                }
                None => println!("file processing failed"),
            };
        }
        Err(err) => println!("error while calling import_xml: {:?}", err),
    }
}

fn import_xml<'a>(file_path: PathBuf) -> Option<(LibcFreer<&'a [u8]>, PathBuf)> {
    if let Some(path) = file_path.to_str() {
        unsafe {
            let now = Instant::now();
            println!(
                "processing file: {:?}",
                file_path.file_name().expect("no file name")
            );

            let c_path = CString::new(path).expect("cstring creation failed");

            let res = cw_import_xml(c_path.as_ptr() as *const u8);
            let mut out_path = PathBuf::new();

            if res.data == null() || res.file_name == null() {
                println!("null ptr from cw");
                return None;
            }
            let slice = std::slice::from_raw_parts(res.data, res.data_len);
            let data = LibcFreer::new(slice, res.data as *mut c_void);

            if let Ok(file_name) = CStr::from_ptr(res.file_name as *const i8).to_str() {
                let file_name = file_name.to_owned();
                libc::free(res.file_name as *mut libc::c_void);
                match file_path.parent() {
                    Some(dir) => {
                        println!("import xml {} took: {:.2?}", file_name, now.elapsed());
                        out_path.push(dir);
                        out_path.push(file_name);
                        return Some((data, out_path));
                    }
                    None => return None,
                }
            }
        }
    }
    return None;
}
