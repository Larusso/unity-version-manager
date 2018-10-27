use std::path::PathBuf;
use super::*;

extern crate winapi;
use self::winapi::um::winver::GetFileVersionInfoA;
use self::winapi::um::winver::GetFileVersionInfoSizeA;
use self::winapi::um::winver::VerQueryValueA;

pub fn get_unity_version(path:PathBuf) -> Option<Version> {
    unsafe {
        None
    }
}
