#[repr(C)]
pub struct RawCwCallRes {
    pub data: *const u8,
    pub data_len: usize,
    pub file_name: *const u8,
}
unsafe impl Send for RawCwCallRes {}
unsafe impl Sync for RawCwCallRes {}

extern "C" {
    pub fn cw_import_xml(path: *const u8) -> RawCwCallRes;
    pub fn gc_collect();
}
