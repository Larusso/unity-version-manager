use std::path::PathBuf;
use super::*;

extern crate winapi;
use self::winapi::um::winver;
use self::winapi::shared::minwindef::DWORD;
use self::winapi::um::winnt::LPCSTR;

pub fn get_unity_version(path:PathBuf) -> Option<Version> {
    let bytes = path.to_string_lossy().into_bytes() + b"\0";
    let cchars = bytes.map_in_place(|b| b as c_char);
    unsafe {
        let mut dummy:DWORD = std::ptr::null_mut();
        let version_size:DWORD = winver::GetFileVersionInfoSize(cchars, &dummy);
        None
    }
}
