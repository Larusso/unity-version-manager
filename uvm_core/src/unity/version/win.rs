use std::path::{PathBuf,Path};
use super::*;

extern crate winapi;
extern crate libc;
extern crate core;

use self::winapi::um::winver;
use self::winapi::shared::minwindef::DWORD;
use self::winapi::shared::minwindef::BYTE;
use self::winapi::shared::ntdef::NULL;
use self::winapi::shared::ntdef::CHAR;
use self::winapi::shared::minwindef::PUINT;
use self::winapi::um::winnt::LPCSTR;
use std::os::raw::c_char;
use std::mem;
use std::fmt;
use std::ffi::{CString, CStr};
use self::winapi::ctypes::c_void;
use self::winapi::ctypes::wchar_t;
use self::winapi::um::memoryapi::VirtualAlloc;
use self::winapi::um::winnt::MEM_COMMIT;
use self::winapi::um::winnt::PAGE_READWRITE;
use self::winapi::shared::basetsd::SIZE_T;
use std::convert::AsRef;
use crate::result::Result;
use crate::error::UvmError;
use std::io;

pub fn read_version_from_path<P : AsRef<Path>>(path:P) -> Result<Version> {
    let path = path.as_ref();
    if !path.exists() {
        return Err(UvmError::IoError(io::Error::new(io::ErrorKind::InvalidInput, format!("Provided Path does not exist. {}", path.display()))))
    }

    if path.is_dir() {
        //check for the `Unity.exe`
        let executable_path = path.join("Editor/Unity.exe");
        let version_string = win_query_version_value(&executable_path, r"\StringFileInfo\040904b0\Unity Version")
            .map_err(|err| ParseVersionError::new("error"))?;
        //let company_name = win_query_version_value(&path,r"\StringFileInfo\040904b0\CompanyName");
        let version_parts:Vec<&str> = version_string.as_str().split('_').collect();
        let version = Version::from_str(version_parts[0])?;
        Ok(version)
    } else {
        Err(UvmError::IoError(io::Error::new(io::ErrorKind::InvalidInput, "Provided Path is not a Unity installation.")))
    }
}

#[derive(Debug)]
pub struct WinVersionError {
    message: String
}

impl WinVersionError {
    fn new(message: &str) -> WinVersionError {
        WinVersionError { message: String::from(message) }
    }
}

impl fmt::Display for WinVersionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "IllegalOperationError {}", self.message)
    }
}

impl Error for WinVersionError {
    fn description(&self) -> &str {
        &self.message[..]
    }
}


fn win_query_version_value(path:&Path, query:&str) -> std::result::Result<String, WinVersionError> {
    let c_path = path.to_str()
                             .and_then(|path| CString::new(path).ok())
                             .ok_or_else(|| {
                                 WinVersionError::new("failed to create CString from path")
                             })?;

    unsafe {
        let dummy: *mut DWORD = std::ptr::null_mut();
        let version_size:DWORD = winver::GetFileVersionInfoSizeA(c_path.as_ptr(), dummy);
        debug!("fetch file version info size for path {:?}", c_path);

        if version_size == 0 {
            return Err(WinVersionError::new("failed to fetch version info size"))
        }

        let mut data = libc::malloc(mem::size_of::<BYTE>() * (version_size as SIZE_T)) as *mut c_void;
        if data.is_null() {
            return Err(WinVersionError::new("failed to allocate memory for version data"))
        }
        let version_info = winver::GetFileVersionInfoA(c_path.as_ptr(), 0, version_size, data);

        if version_info == 0 {
            libc::free(data as *mut libc::c_void);
            return Err(WinVersionError::new(&format!("failed fetch version info for file {:?}", c_path)))
        }

        let mut i_unity_version_len:u32 = 0;
        let raw = &mut i_unity_version_len as PUINT;

        let mut pv_unity_version: *mut CHAR = std::ptr::null_mut();
        let mut pv_unity_version_ptr: *mut c_void = &mut pv_unity_version as *mut _ as *mut c_void;
        let query_string = CString::new(query)
            .map_err(|_| {
                WinVersionError::new("failed to create CString from query")
            })?;

        info!("start query version info {:?}", query_string);
        let version_result = winver::VerQueryValueA(data, query_string.as_ptr(),&mut pv_unity_version_ptr, raw);
        if version_result == 0 {
            libc::free(data as *mut libc::c_void);
            return Err(WinVersionError::new(&format!("failed to query version information from {:?} with query {:?}", c_path, query_string)))
        }

        let result = CStr::from_ptr(pv_unity_version_ptr as *const i8);
        result.to_str()
            .map(|string| String::from(string))
            .map_err(|err| WinVersionError::new("Unable to create UTF8 string"))
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
