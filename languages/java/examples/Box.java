public class Box<T> {
    private T inner;

    public Box(T inner) {
        this.inner = inner;
    }

    public T get() {
        return this.inner;
    }
}
