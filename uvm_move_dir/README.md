uvm_move_dir
============

Opinionated directory move operations.
The default `rename` operation from [`std::fs::rename`] doesn't work for some usecases `uvm` has during
installation of certain modules. The special `move_dir` function in this crate allows:

* to move files into the same path.
* rename directories

A special windows invocation of [`MoveFileExW`] without the `MOVEFILE_REPLACE_EXISTING` flag, which [`std::fs::rename`] sets,
allows moving of directories also under windows.

[`MoveFileExW`]: https://docs.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-movefileexw
[`std::fs::rename`]: https://doc.rust-lang.org/std/fs/fn.rename.html
