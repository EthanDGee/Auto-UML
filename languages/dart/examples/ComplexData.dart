import 'User.dart';

class ComplexData {
  List<int>? raw_bytes;
  List<String> items = [];
  Map<String, int> config = {};
  User? owner;

  void process(String mode) {
    // ...
  }

  static ComplexData create() {
    return ComplexData();
  }
}

class Metadata {
  String key = "";
  String value = "";

  void log() {}
}
