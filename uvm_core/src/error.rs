error_chain! {
    types {
        UvmError, UvmErrorKind, ResultExt, Result;
    }

    foreign_links {
        Fmt(::std::fmt::Error);
        Io(::std::io::Error);
        NetworkError(reqwest::Error);
        VersionError(crate::unity::VersionError);
        HubError(crate::unity::hub::UvmHubError);
    }

    errors {
        IllegalOperation(t: String) {
            description("illegal operation"),
            display("illegal operation: '{}'", t),
        }
        ManifestReadError {
            description("failed to parse version manifest")
        }
    }
}
