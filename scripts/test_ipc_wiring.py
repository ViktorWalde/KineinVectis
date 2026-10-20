"""A identidade da aplicação não é um pedido IPC da ponte."""
import tempfile
from importlib import import_module
import unittest
from pathlib import Path
from unittest.mock import patch

wiring = import_module("verificar_fiacao_ipc")


class ClientMethodsTests(unittest.TestCase):
    def test_only_bridge_and_cli_literals_are_client_methods(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            ui = root / "ui/src"
            cli = root / "crates/kinein-cli/src"
            ui.mkdir(parents=True)
            cli.mkdir(parents=True)
            (ui / "main.cpp").write_text('QStringLiteral("io.github.example.App");\n')
            (ui / "core_client_probe.cpp").write_text(
                '#include "not.a.method"\n'
                'sendRequest(QStringLiteral("probe.list"), {});\n'
                'sendRequest(QStringLiteral("unknown.request"), {});\n'
            )
            (cli / "main.rs").write_text('request("job.list");\n')
            with patch.object(wiring, "RAIZ", root):
                self.assertEqual(wiring.client_methods(),
                                 {"probe.list", "unknown.request", "job.list"})


if __name__ == "__main__":
    unittest.main()
