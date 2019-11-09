use error_chain::error_chain;

error_chain! {
    foreign_links {
        Fmt(::std::fmt::Error);
        Io(::std::io::Error);
        NetworkError(::reqwest::Error);
        ZipError(::zip::result::ZipError);
    }

    errors {
        ChecksumVerificationFailed {
            description("checksum verification failed")
        }
        ManifestLoadFailed

        UnknownInstallerType(provided: String, expected: String) {
            description("unknown installer"),
            display("unknown installer {}. Expect {}", provided, expected),
        }

        MissingDestination(installer: String) {
            description("missing destination"),
            display("missing destination for {}", installer),
        }

        MissingCommand(installer: String) {
            description("missing cmd"),
            display("missing command for {}", installer),
        }

        InstallerCreationFailed {
            description("unable to create installer"),
        }
    }
}
