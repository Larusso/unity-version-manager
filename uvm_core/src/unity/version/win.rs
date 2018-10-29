use std::path::PathBuf;
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


pub fn get_unity_version(path:PathBuf) -> Option<Version> {

    let cchars = CString::new(path.to_str().unwrap()).unwrap();

    unsafe {
        let dummy: *mut DWORD = std::ptr::null_mut();
        let version_size:DWORD = winver::GetFileVersionInfoSizeA(cchars.as_ptr(), dummy);

        if version_size == 0 {
            None
        } else {
            let mut data: *mut BYTE = libc::malloc(mem::size_of::<BYTE>()) as *mut BYTE;
            if data.is_null() {
                panic!("failed to allocate memory");
            }
            let data_ptr: *mut c_void = &mut data as *mut _ as *mut c_void;
            let version_info = winver::GetFileVersionInfoA(cchars.as_ptr(), 0, version_size, data_ptr);
            if version_info == 0 {
                libc::free(data as *mut libc::c_void);
                None
            } else {
                let mut i_unity_version_len:usize = 0;
                let raw = &mut i_unity_version_len as *mut usize;

                let mut pv_unity_version: *mut CHAR = std::ptr::null_mut();
                let mut pv_unity_version_ptr: *mut c_void = &mut pv_unity_version as *mut _ as *mut c_void;
                let q = CString::new(r"\StringFileInfo\040904b0\Unity Version").unwrap();
                let version_result = winver::VerQueryValueA(data_ptr, q.as_ptr(),&mut pv_unity_version_ptr, raw as *mut u32);
                if version_result == 0 {
                    libc::free(data as *mut libc::c_void);
                    None
                } else {
                    let i = i_unity_version_len;
                    //copy_to_nonoverlapping
                    let mut ret = libc::malloc(mem::size_of::<CHAR>() * i) as *mut CHAR;
                    //char *ret = new char[iUnityVersionLen];
                    pv_unity_version.copy_to_nonoverlapping(ret, mem::size_of::<CHAR>() * i);
                    libc::free(data as *mut libc::c_void);

                    let c_ret = CStr::from_ptr(ret);
                    let v = Version::from_str(c_ret.to_str().unwrap()).unwrap();
                    Some(v)
                    //memcpy(ret, pvUnityVersion, iUnityVersionLen * sizeof(char));
                    //delete[] data;
                    //return ret;
                    //None
                }
            }
        }
    }
}
