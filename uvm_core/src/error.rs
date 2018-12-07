error_chain! {
    types {
        UvmError, UvmErrorKind, ResultExt, Result;
    }

    links {
        VersionError(::unity::UvmVersionError, ::unity::UvmVersionErrorKind);
        HubError(::unity::hub::UvmHubError, ::unity::hub::UvmHubErrorKind);
        InstallError(::install::UvmInstallError, ::install::UvmInstallErrorKind);
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
