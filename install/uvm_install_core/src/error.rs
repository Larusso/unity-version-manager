use error_chain::error_chain;

error_chain! {
    types {
        UvmInstallError, UvmInstallErrorKind, ResultExt, Result;
    }

    foreign_links {
        Fmt(::std::fmt::Error);
        Io(::std::io::Error);
        NetworkError(::reqwest::Error);
    }

    errors {
        ChecksumVerificationFailed {
            description("checksum verification failed")
        }
        ManifestLoadFailed
    }
}
