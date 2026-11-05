#!/usr/bin/env python3
"""Exercise environment UI and buffer guards with the project's Qt runner."""

import argparse
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parent.parent
SOURCE = Path("ui/qml/components/KvSelect.qml")
CASES = [
    ("selected_path", "edited(normalizedChoices[index].value);",
     "edited(normalizedChoices[index].label);"),
    ("custom_value", "property bool allowCustom: true", "property bool allowCustom: false"),
    ("empty_value", 'edited("");', 'edited(normalizedChoices[0].value);'),
    ("hidden_menu", "onVisibleChanged: if (!visible) { close(false); customRequested = false; }",
     "onVisibleChanged: if (visible && !visible) { close(false); customRequested = false; }"),
    ("busy_menu", "if (!enabled || busy || !visible || !root.Window.window) return;",
     "if (!enabled || !visible || !root.Window.window) return;"),
    ("refresh_selection", "onChoicesChanged: close(false)",
     'onChoicesChanged: { close(false); if (normalizedChoices.length) edited(normalizedChoices[0].value); }'),
]
CHECKS = [(label, SOURCE, before, after, "tst_choice_fields")
          for label, before, after in CASES] + [
    ("reopen_model", Path("ui/qml/editor/EditorDocumentController.qml"),
     "const alreadyOpen = openDocuments.docIdForPath(path);",
     "const alreadyOpen = openDocuments.applyDiskContent(path, content, false);",
     "tst_editor_reopen_buffer"),
    ("reopen_surface", Path("ui/qml/editor/EditorDocumentController.qml"),
     "selectDocument(alreadyOpen);",
     "selectDocument(alreadyOpen); if (surfaceBridge !== null) surfaceBridge.setText(content);",
     "tst_editor_reopen_buffer"),
]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--logs", type=Path, default=ROOT / "build/choice-field-mutations")
    args = parser.parse_args()
    args.logs.mkdir(parents=True, exist_ok=True)
    qt_bins = (ROOT / "build/dev-local/kinein-qt-bins.txt").read_text().strip()
    env = {**os.environ, "KINEIN_QML_RUNNER": str(Path(qt_bins) / "qml"),
           "KINEIN_QML_TEST": "tst_choice_fields", "KINEIN_QML_JOBS": "2"}
    with tempfile.TemporaryDirectory(prefix="kinein-choice-mutations-") as temporary:
        checkout = Path(temporary)
        for relative in ["ui/qml", "scripts/qml-harness"]:
            shutil.copytree(ROOT / relative, checkout / relative)
        shutil.copy2(ROOT / "scripts/verificar-qml-logica.sh", checkout / "scripts")

        def run(label, test):
            env["KINEIN_QML_TEST"] = test
            result = subprocess.run(["bash", "scripts/verificar-qml-logica.sh"],
                                    cwd=checkout, env=env, text=True, capture_output=True,
                                    timeout=90, check=False)
            output = result.stdout + result.stderr
            (args.logs / f"{label}.log").write_text(output)
            return result.returncode, output

        for test in sorted({entry[4] for entry in CHECKS}):
            code, _ = run("baseline-" + test, test)
            if code:
                raise SystemExit("Baseline failed; no mutation verdict is valid")
        for label, source, before, after, test in CHECKS:
            original = (checkout / source).read_text()
            if original.count(before) != 1:
                raise SystemExit(f"Mutation {label}: expected one source match")
            (checkout / source).write_text(original.replace(before, after))
            code, output = run(label, test)
            if code == 0 or "GUARD:" not in output or re.search(
                r"SyntaxError|TypeError|ReferenceError|Unable to assign|is not a type|TIMEOUT", output
            ):
                raise SystemExit(f"Mutation {label} survived or did not load; inspect its log")
            print(f"killed: {label}")
            (checkout / source).write_text(original)
    print(f"{len(CHECKS)}/{len(CHECKS)} mutations killed")


if __name__ == "__main__":
    main()
