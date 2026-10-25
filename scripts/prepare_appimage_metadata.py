#!/usr/bin/env python3
"""Deriva a identidade do AppImage de desenvolvimento dos modelos instalados."""

import argparse
from pathlib import Path
import xml.etree.ElementTree as ET

STABLE_ID = "io.github.viktorwalde.KineinVectis"
DEVELOPMENT_ID = STABLE_ID + ".Development"
DEVELOPMENT_NAME = "Kinein Vectis (desenvolvimento)"


def prepare(appdir: Path, version: str, date: str) -> None:
    """Muda so' a identidade do canal, sem duplicar a descricao do produto."""
    desktop = appdir / "usr/share/applications" / (STABLE_ID + ".desktop")
    text = desktop.read_text()
    text = text.replace("Name=Kinein Vectis\n", f"Name={DEVELOPMENT_NAME}\n")
    text = text.replace("Icon=kinein-vectis\n", f"Icon={DEVELOPMENT_ID}\n")
    desktop.with_name(DEVELOPMENT_ID + ".desktop").write_text(text)
    desktop.unlink()

    icon = appdir / "usr/share/icons/hicolor/512x512/apps/kinein-vectis.png"
    icon.rename(icon.with_name(DEVELOPMENT_ID + ".png"))
    metadata = appdir / "usr/share/metainfo" / (STABLE_ID + ".appdata.xml")
    tree = ET.parse(metadata)
    root = tree.getroot()
    root.find("id").text = DEVELOPMENT_ID
    root.find("name").text = DEVELOPMENT_NAME
    root.find("launchable").text = DEVELOPMENT_ID + ".desktop"
    releases = root.find("releases")
    releases.clear()
    ET.SubElement(releases, "release", version=version, date=date)
    tree.write(metadata.with_name(DEVELOPMENT_ID + ".appdata.xml"), encoding="utf-8", xml_declaration=True)
    metadata.unlink()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("appdir", type=Path)
    parser.add_argument("version")
    parser.add_argument("date")
    args = parser.parse_args()
    if "-dev" not in args.version:
        parser.error("a versao de desenvolvimento precisa de sufixo -dev")
    prepare(args.appdir, args.version, args.date)


if __name__ == "__main__":
    main()
