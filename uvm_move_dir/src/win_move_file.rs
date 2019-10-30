use std::io;
use std::path::Path;

pub fn rename<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> io::Result<()> {
    use winapi::um::winbase;
    use winapi::um::errhandlingapi;
    use widestring::{U16CString};

    let from = from.as_ref();
    let to = to.as_ref();

    let from = U16CString::from_os_str(from).map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
    let to = U16CString::from_os_str(to).map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

    unsafe {
        let success = winbase::MoveFileW(from.as_ptr(), to.as_ptr());
        if success == 0 {
            let error_code = errhandlingapi::GetLastError();
            Err(io::Error::from_raw_os_error(error_code as i32))
        } else {
            Ok(())
        }
    }
}
