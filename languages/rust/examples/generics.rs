struct Box<T> {
    inner: T,
}

impl<T> Box<T> {
    fn new(inner: T) -> Box<T> {
        Box { inner }
    }

    fn get(self) -> T {
        self.inner
    }
}
