error_chain! {
    types {
        UvmError, UvmErrorKind, ResultExt, Result;
    }

    links {
        VersionError(crate::unity::UvmVersionError, crate::unity::UvmVersionErrorKind);
        HubError(crate::unity::hub::UvmHubError, crate::unity::hub::UvmHubErrorKind);
    }

    foreign_links {
        Fmt(::std::fmt::Error);
        Io(::std::io::Error);
        NetworkError(reqwest::Error);
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
