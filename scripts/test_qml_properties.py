"""Prova handlers locais em raízes derivadas sem aceitar bindings errados."""
import importlib.util
from pathlib import Path
import tempfile
import unittest

REPO_ROOT = Path(__file__).resolve().parent.parent
spec = importlib.util.spec_from_file_location(
    "qml_properties", REPO_ROOT / "scripts" / "verificar_qml_propriedades.py")
scanner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(scanner)

BASE_SOURCE = """Rectangle {
    property string inheritedText: ""
}
"""


class LocalHandlers(unittest.TestCase):
    def check_source(self, source):
        with tempfile.TemporaryDirectory(prefix="kinein-qml-properties-", dir=REPO_ROOT) as directory:
            path = Path(directory) / "DerivedRoot.qml"
            path.write_text(source)
            types = {"BaseRoot": scanner.interface(BASE_SOURCE),
                     "DerivedRoot": scanner.interface(source) | scanner.interface(BASE_SOURCE)}
            return scanner.check_file(path, types)

    def test_local_property_signal_and_alias_are_valid(self):
        source = """BaseRoot {
    property string payload: ""
    property alias value: field.text
    signal submitted()
    inheritedText: "inherited"
    onPayloadChanged: {}
    onValueChanged: {}
    onSubmitted: {}
}
"""
        self.assertEqual(self.check_source(source), [])

    def test_misspelled_local_handler_is_rejected(self):
        source = """BaseRoot {
    property string payload: ""
    onPaylodChanged: {}
}
"""
        errors = self.check_source(source)
        self.assertEqual(len(errors), 1)
        self.assertIn("onPaylodChanged", errors[0])

    def test_native_qt_root_keeps_the_qmllint_boundary(self):
        self.assertEqual(self.check_source("""Timer {
    interval: 100
    repeat: true
    onTriggered: {}
}
"""), [])

    def test_outer_property_does_not_belong_to_nested_instance(self):
        source = """BaseRoot {
    property string payload: ""
    onPayloadChanged: {}
    BaseRoot {
        onPayloadChanged: {}
    }
}
"""
        errors = self.check_source(source)
        self.assertEqual(len(errors), 1)
        self.assertIn(":5:", errors[0])


if __name__ == "__main__":
    unittest.main()
