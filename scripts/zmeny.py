#!/usr/bin/env python3
"""Přehled změn CHANGELOG.md (česky) a CHANGELOG.en.md (anglicky).

Příkazy:
  kontrola [--tag vX.Y.Z]   ověří, že oba soubory mají stejné verze ve stejném
                            pořadí, sekci Nevydáno / Unreleased, odkazy a popis
                            aktuální verze z Cargo.toml; s --tag navíc, že tag
                            odpovídá verzi programu (spouští CI a workflow Vydání)
  poznamky X.Y.Z            vypíše poznámky k vydání (česká a anglická sekce)

[en] Changelog CHANGELOG.md (Czech) and CHANGELOG.en.md (English).

Commands:
  kontrola [--tag vX.Y.Z]   checks that both files have the same versions in the
                            same order, an Unreleased section, links and an entry
                            for the current version from Cargo.toml; with --tag
                            also that the tag matches the program version (run by
                            CI and the release workflow)
  poznamky X.Y.Z            prints the release notes (Czech and English section)
"""

import os
import re
import sys
from pathlib import Path

# Kořen repozitáře.
# [en] Repository root.
ROOT = Path(__file__).resolve().parent.parent

# Soubory přehledu a název sekce s nevydanými změnami v každém z nich.
# [en] Changelog files and the name of the unreleased section in each of them.
SOUBORY = {
    "cs": (ROOT / "CHANGELOG.md", "Nevydáno"),
    "en": (ROOT / "CHANGELOG.en.md", "Unreleased"),
}

# Nadpis verze: „## [0.3.1] – 2026-11-21“ nebo „## [Nevydáno]“.
# [en] Version heading: "## [0.3.1] – 2026-11-21" or "## [Unreleased]".
NADPIS = re.compile(r"^## \[([^\]]+)\](?: – (\d{4}-\d{2}-\d{2}))?\s*$")

# Číslo verze ve tvaru X.Y.Z.
# [en] Version number in the X.Y.Z form.
VERZE = re.compile(r"^\d+\.\d+\.\d+$")


def verze_programu():
    """Verze z [workspace.package] v Cargo.toml.

    [en] Version from [workspace.package] in Cargo.toml.
    """
    text = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
    sekce = text.split("[workspace.package]", 1)[1]
    return re.search(r'^version\s*=\s*"([^"]+)"', sekce, re.M).group(1)


def sekce(cesta):
    """Rozdělí soubor na sekce verzí: seznam (název, datum, řádky obsahu).

    Sekce končí dalším nadpisem úrovně 2 (např. „## Plán dalších verzí“).

    [en] Splits the file into version sections: list of (name, date, content lines).

    A section ends at the next level 2 heading (e.g. "## Roadmap").
    """
    vysledek = []
    aktualni = None
    for radek in cesta.read_text(encoding="utf-8").splitlines():
        if radek.startswith("## "):
            m = NADPIS.match(radek)
            aktualni = [m.group(1), m.group(2), []] if m else None
            if aktualni:
                vysledek.append(aktualni)
        elif aktualni is not None:
            aktualni[2].append(radek)
    return [tuple(s) for s in vysledek]


def kontrola(tag=None):
    """Zkontroluje oba soubory; vrátí seznam chyb.

    [en] Checks both files; returns a list of errors.
    """
    chyby = []
    program = verze_programu()
    if tag is not None and tag != f"v{program}":
        chyby.append(
            f"tag {tag} neodpovídá verzi programu {program} v Cargo.toml "
            f"(tag má být v{program}) / [en] tag does not match the program version"
        )
    seznamy = {}
    for jazyk, (cesta, nevydano) in SOUBORY.items():
        if not cesta.exists():
            chyby.append(f"chybí soubor {cesta.name} / [en] missing file")
            continue
        text = cesta.read_text(encoding="utf-8")
        s = sekce(cesta)
        if not s or s[0][0] != nevydano:
            chyby.append(
                f"{cesta.name}: první sekce musí být „## [{nevydano}]“ "
                f"/ [en] the first section must be the unreleased one"
            )
        verze = [(n, d) for n, d, _ in s if n != nevydano]
        for nazev, datum, obsah in s:
            if nazev == nevydano:
                continue
            if not VERZE.match(nazev) or datum is None:
                chyby.append(
                    f"{cesta.name}: nadpis „{nazev}“ musí mít tvar „## [X.Y.Z] – RRRR-MM-DD“ "
                    f"/ [en] heading must look like"
                )
            if not any(r.strip() for r in obsah):
                chyby.append(f"{cesta.name}: verze {nazev} je prázdná / [en] version is empty")
        for nazev, _, _ in s:
            if f"\n[{nazev}]: " not in text:
                chyby.append(
                    f"{cesta.name}: chybí odkaz „[{nazev}]: …“ na konci souboru "
                    f"/ [en] missing link definition"
                )
        if program not in [n for n, _ in verze]:
            chyby.append(
                f"{cesta.name}: chybí sekce „## [{program}] – datum“ pro verzi programu "
                f"z Cargo.toml / [en] missing section for the program version"
            )
        seznamy[jazyk] = verze
    if len(seznamy) == 2 and seznamy["cs"] != seznamy["en"]:
        chyby.append(
            "CHANGELOG.md a CHANGELOG.en.md nemají stejné verze a data ve stejném pořadí: "
            f"{seznamy['cs']} × {seznamy['en']} / [en] the files differ in versions or dates"
        )
    return chyby


def poznamky(verze):
    """Poznámky k vydání: česká a anglická sekce dané verze.

    [en] Release notes: the Czech and English section of the given version.
    """
    casti = []
    for jazyk, nadpis in (("cs", "Česky"), ("en", "English")):
        cesta = SOUBORY[jazyk][0]
        obsah = next((o for n, _, o in sekce(cesta) if n == verze), None)
        if obsah is None:
            sys.exit(f"{cesta.name}: verze {verze} nenalezena / [en] version not found")
        # Nadpisy sekce o úroveň níž, aby byly pod nadpisem jazyka.
        # [en] Section headings one level lower so they sit under the language heading.
        text = "\n".join("#" + r if r.startswith("### ") else r for r in obsah).strip()
        casti.append(f"## {nadpis}\n\n{text}")
    return "\n\n".join(casti) + "\n"


def main():
    """Zpracuje příkazovou řádku.

    [en] Handles the command line.
    """
    argumenty = sys.argv[1:]
    if argumenty[:1] == ["kontrola"]:
        tag = None
        if argumenty[1:2] == ["--tag"] and len(argumenty) > 2:
            tag = argumenty[2]
        chyby = kontrola(tag)
        for chyba in chyby:
            print(f"::error::{chyba}" if "GITHUB_ACTIONS" in os.environ else chyba)
        if chyby:
            sys.exit(1)
        print(f"Přehled změn: v pořádku (verze {verze_programu()}).")
    elif argumenty[:1] == ["poznamky"] and len(argumenty) == 2:
        sys.stdout.write(poznamky(argumenty[1].removeprefix("v")))
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main()
