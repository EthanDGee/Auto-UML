class Box<T> {
  T inner;

  Box(this.inner);

  T get() {
    return this.inner;
  }
}
