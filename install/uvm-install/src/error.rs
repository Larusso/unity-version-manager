use error_chain::error_chain;
error_chain! {
    links {
        UvmError(uvm_core::error::UvmError, uvm_core::error::UvmErrorKind);
        UvmInstallCoreError(uvm_install_core::error::Error, uvm_install_core::error::ErrorKind);
    }

    foreign_links {
        Fmt(::std::fmt::Error);
        Io(::std::io::Error);
    }
}