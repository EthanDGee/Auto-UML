import java.util.List;
import java.util.Map;

public class ComplexData {
    private byte[] raw_bytes;
    private List<String> items;
    private Map<String, Integer> config;
    private User owner;

    public void process(String mode) {
        // ...
    }

    public static ComplexData create() {
        return new ComplexData();
    }

    private class Metadata {
        public String key;
        public String value;

        public void log() {}
    }
}
