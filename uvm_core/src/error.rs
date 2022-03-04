error_chain! {
    types {
        UvmError, UvmErrorKind, ResultExt, Result;
    }

    links {
        HubError(crate::unity::hub::UvmHubError, crate::unity::hub::UvmHubErrorKind);
    }

    foreign_links {
        Fmt(::std::fmt::Error);
        Io(::std::io::Error);
        NetworkError(reqwest::Error);
        VersionError(crate::unity::VersionError);
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
