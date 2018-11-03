use std::path::{PathBuf,Path};
use super::*;

extern crate winapi;
extern crate libc;
use self::winapi::um::winver;
use self::winapi::shared::minwindef::DWORD;
use self::winapi::shared::minwindef::BYTE;
use self::winapi::shared::ntdef::NULL;
use self::winapi::shared::ntdef::CHAR;
use self::winapi::shared::minwindef::PUINT;
use self::winapi::um::winnt::LPCSTR;
use std::os::raw::c_char;
use std::mem;
use std::ffi::{CString, CStr};
use self::winapi::ctypes::c_void;
use self::winapi::ctypes::wchar_t;
use self::winapi::um::memoryapi::VirtualAlloc;
use self::winapi::um::winnt::MEM_COMMIT;
use self::winapi::um::winnt::PAGE_READWRITE;
use self::winapi::shared::basetsd::SIZE_T;
use std::convert::AsRef;


pub fn get_unity_version(path:PathBuf) -> Option<Version> {
    let version_string = win_query_version_value(path)?;
    let version_parts:Vec<&str> = version_string.as_str().split('_').collect();
    let version = Version::from_str(version_parts[0]).unwrap();
    Some(version)
}

fn win_query_version_value<P : AsRef<Path>>(path:P) -> Option<String> {
    let str_path = path.as_ref().to_str()?;
    let c_path = CString::new(str_path).ok()?;

    unsafe {
        let dummy: *mut DWORD = std::ptr::null_mut();
        let version_size:DWORD = winver::GetFileVersionInfoSizeA(c_path.as_ptr(), dummy);
        debug!("fetch file version info size for path {:?}", c_path);

        if version_size == 0 {
            return None
        }

        let mut data = libc::malloc(mem::size_of::<BYTE>() * (version_size as SIZE_T)) as *mut c_void;
        if data.is_null() {
            error!("failed to allocate memory");
            return None
        }
        let version_info = winver::GetFileVersionInfoA(c_path.as_ptr(), 0, version_size, data);

        if version_info == 0 {
            libc::free(data as *mut libc::c_void);
            error!("failed fetch version info for file {:?}", c_path);
            return None
        }

        let mut i_unity_version_len:u32 = 0;
        let raw = &mut i_unity_version_len as PUINT;

        let mut pv_unity_version: *mut CHAR = std::ptr::null_mut();
        let mut pv_unity_version_ptr: *mut c_void = &mut pv_unity_version as *mut _ as *mut c_void;
        let query_string = CString::new(r"\StringFileInfo\040904b0\Unity Version").ok()?;

        info!("start query version info {:?}", query_string);
        let version_result = winver::VerQueryValueA(data, query_string.as_ptr(),&mut pv_unity_version_ptr, raw);
        if version_result == 0 {
            libc::free(data as *mut libc::c_void);
            error!("failed to query version information from {:?} with query {:?}", c_path, query_string);
            return None
        }

        let result = CStr::from_ptr(pv_unity_version_ptr as *const i8);
        result.to_str().map(|string| String::from(string)).ok()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path,PathBuf};
    #[test]
    fn check_win() {
        let path:PathBuf = Path::new(r"C:\Program Files\Unity\Editor\Unity.exe").to_path_buf();
        let v = get_unity_version(path);
        let version = Version::from_str("5.5.5f1").unwrap();
        assert_eq!(v, Some(version));
    }
}
