class ComplexData {
    private val raw_bytes: ByteArray = ByteArray(0)
    private val items: List<String> = emptyList()
    private val config: Map<String, Int> = emptyMap()
    private val owner: User = User()

    fun process(mode: String) {
        // ...
    }

    fun create(): ComplexData {
        return ComplexData()
    }

    private class Metadata {
        var key: String = ""
        var value: String = ""

        fun log() {}
    }
}
