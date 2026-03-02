struct ComplexData {
    raw_bytes: Vec<u8>,
    metadata: std::collections::HashMap<String, String>,
}

impl ComplexData {
    fn process(&self, mode: String) -> Result<bool, Error> {
        true
    }

    fn update<T>(&mut self, value: T) where T: Into<String> {
        // ...
    }
}

mod internal {
    struct Secret {
        key: String,
    }

    impl Secret {
        fn hide(&self) {}
    }
}
