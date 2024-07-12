use std::path::PathBuf;

mod convert;
mod ffi;

pub enum Type {
    TextureDict,
    Xml,
    Rage(String),
}

pub fn cw_gc() {
    unsafe { ffi::gc_collect() }
}

pub async fn process_file(f_type: Type, path: PathBuf) {
    match f_type {
        Type::Xml => convert::xml(path).await,
        Type::Rage(_) => todo!(),
        Type::TextureDict => todo!(),
    };
}
