error_chain! {
    types {
        ParseComponentError, ParseComponentErrorKind, ResultExt, Result;
    }

    errors {
        Unsupported(t: String) {
            description("unsupported component"),
            display("unsupported component: '{}'", t),
        }

        UnsupportedCategory(t: String) {
            description("unsupported component category"),
            display("unsupported component category: '{}'", t),
        }
    }
}
