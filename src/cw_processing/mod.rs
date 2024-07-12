use std::path::PathBuf;

mod export;
mod ffi;
mod import;

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
        Type::Xml => import::xml(path).await,
        Type::Rage(_) => todo!(),
        Type::TextureDict => export::texture_dict(path).await,
    };
}
