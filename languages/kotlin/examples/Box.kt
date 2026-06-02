class Box<T> {
    private val inner: T

    constructor(inner: T) {
        this.inner = inner
    }

    fun get(): T {
        return this.inner
    }
}
