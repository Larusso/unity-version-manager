use error_chain::error_chain;
use uvm_core::install::error as install_error;
error_chain! {
    links {
        UvmError(uvm_core::error::UvmError, uvm_core::error::UvmErrorKind);
        UvmInstallError(install_error::UvmInstallError, install_error::UvmInstallErrorKind);
    }

    foreign_links {
        Fmt(::std::fmt::Error);
        Io(::std::io::Error);
    }
}
