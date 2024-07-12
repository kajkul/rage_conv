mod args;
mod cw_processing;
mod libc_freer;

use core::time;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{self, Arc, Mutex},
    time::Duration,
};

use args::{Args, Mode, ARGS};
use clap::Parser;
use notify::{EventKind, RecursiveMode, Watcher};
use tokio::select;

extern crate windows;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = ARGS.get_or_init(|| Args::parse());

    let (tx, rx) = sync::mpsc::channel::<(cw_processing::Type, PathBuf)>();
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
                            let ext = ending.to_str().expect("could not convert extention to str");
                            match ext {
                                "xml" => match args.mode {
                                    Mode::Universal | Mode::Xml => {
                                        tx.send((cw_processing::Type::Xml, path)).unwrap()
                                    }
                                    _ => (),
                                },
                                "pso" | "rbf" | "rel" | "ynd" | "ynv" | "ycd" | "ybn" | "ydr"
                                | "ydd" | "yft" | "ypt" | "yld" | "yed" | "ywr" | "yvr" | "awc"
                                | "fxc" | "dat" | "ypdb" | "yfd" | "mrf" => match args.mode {
                                    Mode::Universal | Mode::Rage => tx
                                        .send((cw_processing::Type::Rage(ext.to_owned()), path))
                                        .unwrap(),
                                    _ => (),
                                },
                                "ytd" => match args.mode {
                                    Mode::Universal | Mode::Rage => {
                                        if args.ytd_export_textures {
                                            tx.send((cw_processing::Type::TextureDict, path))
                                                .unwrap();
                                        }
                                    }
                                    _ => (),
                                },
                                _ => (),
                            }
                        }
                    })
                }
                _ => (),
            },
            Err(e) => println!("watch error: {:?}", e),
        })?;

    watcher.watch(
        Path::new(&args.input_dir),
        match args.recursive {
            true => RecursiveMode::Recursive,
            false => RecursiveMode::NonRecursive,
        },
    )?;

    // Kind of a hacky way to force C# runtime to do gc
    tokio::spawn(async {
        loop {
            tokio::time::sleep(time::Duration::from_secs(30)).await;
            cw_processing::cw_gc()
        }
    });

    for (f_type, path) in rx {
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
                  cw_processing::process_file(f_type, path).await
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
