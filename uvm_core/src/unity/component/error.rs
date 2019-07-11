error_chain! {
    types {
        ParseComponentError, ParseComponentErrorKind, ResultExt, Result;
    }

    errors {
        Unsupported(t: String) {
            description("unsupported component"),
            display("unsupported component: '{}'", t),
        }
    }
}
