#!/usr/bin/env python3
"""Dokumentace formátů souborů v docs/formaty (projekt .logicflow, knihovna .lflib).

Příkazy:
  kontrola    ověří, že JSON Schema jsou platná, že jim odpovídají vzorové
              knihovny (docs/knihovny), obvody nápovědy (docs/napoveda/obvody)
              a příklady ze specifikací; že historie formátů v docs/formaty/README.md
              má řádek pro každou verzi z CHANGELOG.md, nevydané verze jsou
              označené, stav dokumentace odpovídá verzi v Cargo.toml a každá
              hodnota „Od verze“ ve specifikacích a ve schématech je v historii
              (spouští CI). Potřebuje balíček jsonschema (pip install jsonschema).

Shodu dokumentace s kódem (druhy součástek, pole, rozsahy) hlídá test
crates/logicflow-core/tests/formaty.rs.

[en] File format documentation in docs/formaty (project .logicflow, library .lflib).

Commands:
  kontrola    checks that the JSON Schemas are valid and that the sample libraries
              (docs/knihovny), help circuits (docs/napoveda/obvody) and the
              examples from the specifications match them; that the format history
              in docs/formaty/README.md has a row for every version in CHANGELOG.md,
              unreleased versions are marked, the documentation status matches the
              version in Cargo.toml and every “Since” value in the specifications and
              schemas is in the history (run by CI). Needs the jsonschema package
              (pip install jsonschema).

The match between the documentation and the code (component kinds, fields, ranges)
is checked by the test crates/logicflow-core/tests/formaty.rs.
"""

import json
import re
import sys
from pathlib import Path

# Kořen repozitáře a složka s dokumentací formátů.
# [en] Repository root and the folder with the format documentation.
ROOT = Path(__file__).resolve().parent.parent
FORMATY = ROOT / "docs" / "formaty"

# Specifikace podle formátu a jazyka.
# [en] Specifications by format and language.
SPECIFIKACE = {
    "logicflow": [FORMATY / "logicflow.md", FORMATY / "logicflow.en.md"],
    "lflib": [FORMATY / "lflib.md", FORMATY / "lflib.en.md"],
}

# Schéma pro každý formát.
# [en] Schema for each format.
SCHEMATA = {
    "logicflow": FORMATY / "logicflow.schema.json",
    "lflib": FORMATY / "lflib.schema.json",
}

# Soubory v repozitáři, které musí odpovídat schématu.
# [en] Files in the repository that must match the schema.
VZORY = {
    "logicflow": sorted((ROOT / "docs" / "napoveda" / "obvody").glob("*.logicflow")),
    "lflib": sorted((ROOT / "docs" / "knihovny").glob("*.lflib")),
}

# Číslo verze X.Y.Z.
# [en] Version number X.Y.Z.
VERZE = re.compile(r"^\d+\.\d+\.\d+$")

# Označení nevydané verze v historii.
# [en] Marking of an unreleased version in the history.
NEVYDANO = re.compile(r"nevydáno|unreleased", re.I)

# Značka příkladu ve specifikaci: <!-- příklad: lflib --> / <!-- example: lflib -->.
# [en] Example marker in a specification.
PRIKLAD = re.compile(r"<!-- (?:příklad|example): (\w+) -->\s*```json\n(.*?)\n```", re.S)


def verze_programu():
    """Verze z [workspace.package] v Cargo.toml.

    [en] Version from [workspace.package] in Cargo.toml.
    """
    text = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
    sekce = text.split("[workspace.package]", 1)[1]
    return re.search(r'^version\s*=\s*"([^"]+)"', sekce, re.M).group(1)


def klic(verze):
    """Verze jako n-tice čísel pro porovnání.

    [en] Version as a tuple of numbers for comparison.
    """
    return tuple(int(c) for c in verze.split("."))


def verze_v_prehledu():
    """Vydané verze z nadpisů CHANGELOG.md („## [0.3.1] – …“).

    [en] Released versions from the CHANGELOG.md headings.
    """
    text = (ROOT / "CHANGELOG.md").read_text(encoding="utf-8")
    return [v for v in re.findall(r"^## \[([^\]]+)\]", text, re.M) if VERZE.match(v)]


def bunky(radek):
    """Buňky řádku tabulky Markdownu bez okrajových svislic.

    [en] Cells of a Markdown table row without the outer pipes.
    """
    return [b.strip() for b in radek.strip().strip("|").split("|")]


def historie(chyby):
    """Řádky historie formátů z README (česká i anglická tabulka): {verze: [text buňky]}.

    [en] Rows of the format history from the README (Czech and English table).
    """
    radky = {}
    text = (FORMATY / "README.md").read_text(encoding="utf-8")
    for nadpis in ("## Historie formátů", "### Format history"):
        if nadpis not in text:
            chyby.append(f"README.md: chybí sekce „{nadpis}“ / [en] section missing")
            continue
        cast = text.split(nadpis, 1)[1].split("\n#", 1)[0]
        for radek in cast.splitlines():
            if not radek.startswith("| ") or radek.startswith("| ---"):
                continue
            prvni = bunky(radek)[0]
            m = re.match(r"(\d+\.\d+\.\d+)", prvni)
            if m:
                radky.setdefault(m.group(1), []).append(prvni)
    return radky


def zkontroluj_historii(chyby):
    """Historie formátů proti přehledu změn a verzi v Cargo.toml.

    [en] Format history against the changelog and the version in Cargo.toml.
    """
    program = verze_programu()
    radky = historie(chyby)
    for v in verze_v_prehledu():
        if len(radky.get(v, [])) < 2:
            chyby.append(
                f"README.md: v historii formátů chybí verze {v} (česky i anglicky) "
                f"/ [en] version {v} missing in the format history"
            )
    for v, prvni in radky.items():
        for bunka in prvni:
            nevydana = klic(v) > klic(program)
            if nevydana and not NEVYDANO.search(bunka):
                chyby.append(
                    f"README.md: verze {v} je novější než {program} – označte ji „nevydáno“ "
                    f"/ „unreleased“ / [en] mark it unreleased"
                )
            if not nevydana and NEVYDANO.search(bunka):
                chyby.append(
                    f"README.md: verze {v} už vyšla (Cargo.toml {program}) – odstraňte "
                    f"„nevydáno“ / [en] already released, remove “unreleased”"
                )
    # Stav dokumentace v přehledu i v záhlaví specifikací.
    # [en] Documentation status in the overview and in the specification headers.
    for cesta in [FORMATY / "README.md", *sum(SPECIFIKACE.values(), [])]:
        stavy = [
            r
            for r in cesta.read_text(encoding="utf-8").splitlines()
            if re.search(r"^\*\*(Stav dokumentace|Documentation status):\*\*|· (stav|status): ", r)
        ]
        if not stavy:
            chyby.append(f"{cesta.name}: chybí stav dokumentace / [en] documentation status missing")
        for radek in stavy:
            m = re.search(r"LogicFlow \**(\d+\.\d+\.\d+)", radek)
            if not m or m.group(1) != program:
                chyby.append(
                    f"{cesta.name}: stav dokumentace neodpovídá verzi {program} z Cargo.toml "
                    f"/ [en] documentation status does not match: {radek.strip()[:80]}"
                )
    return radky


def zkontroluj_verze_od(radky, schemata, chyby):
    """Každá hodnota „Od verze“ ve specifikacích a x-since ve schématech je v historii.

    [en] Every “Since” value in the specifications and x-since in the schemas is in the history.
    """
    znam = set(radky)
    for cesta in sum(SPECIFIKACE.values(), []):
        for radek in cesta.read_text(encoding="utf-8").splitlines():
            if radek.startswith("| ") and not radek.startswith("| ---"):
                for bunka in bunky(radek):
                    if VERZE.match(bunka) and bunka not in znam:
                        chyby.append(
                            f"{cesta.name}: verze {bunka} není v historii formátů "
                            f"/ [en] version not in the format history"
                        )

    def projdi(hodnota, cesta):
        # Anotace x-since je text nebo objekt {hodnota: verze}.
        # [en] The x-since annotation is a string or an object {value: version}.
        if isinstance(hodnota, dict):
            for k, v in hodnota.items():
                if k in ("x-since", "x-builtin"):
                    for verze in [v] if isinstance(v, str) else v.values():
                        if verze not in znam:
                            chyby.append(
                                f"{cesta}: x-since {verze} není v historii formátů "
                                f"/ [en] not in the format history"
                            )
                else:
                    projdi(v, cesta)
        elif isinstance(hodnota, list):
            for v in hodnota:
                projdi(v, cesta)

    for nazev, schema in schemata.items():
        projdi(schema, SCHEMATA[nazev].name)


def zkontroluj_schemata(schemata, chyby):
    """Platnost schémat a shoda vzorových souborů a příkladů se schématy.

    [en] Validity of the schemas and the match of sample files and examples with them.
    """
    try:
        from jsonschema import Draft202012Validator
        from referencing import Registry, Resource
    except ImportError:
        sys.exit(
            "Chybí balíček jsonschema: pip install jsonschema "
            "/ [en] the jsonschema package is missing: pip install jsonschema"
        )
    registr = Registry().with_resources(
        (s["$id"], Resource.from_contents(s)) for s in schemata.values()
    )
    validatory = {}
    for nazev, schema in schemata.items():
        Draft202012Validator.check_schema(schema)
        validatory[nazev] = Draft202012Validator(schema, registry=registr)

    def over(nazev, popis, text):
        # Jeden soubor nebo příklad proti schématu svého formátu.
        # [en] One file or example against the schema of its format.
        try:
            data = json.loads(text)
        except json.JSONDecodeError as e:
            chyby.append(f"{popis}: není JSON / [en] not JSON: {e}")
            return
        for e in sorted(validatory[nazev].iter_errors(data), key=lambda e: list(e.path)):
            misto = "/".join(str(p) for p in e.absolute_path)
            chyby.append(f"{popis}: {misto}: {e.message[:200]}")

    for nazev, soubory in VZORY.items():
        for cesta in soubory:
            over(nazev, cesta.relative_to(ROOT), cesta.read_text(encoding="utf-8"))
    for nazev, cesty in SPECIFIKACE.items():
        for cesta in cesty:
            priklady = PRIKLAD.findall(cesta.read_text(encoding="utf-8"))
            if not priklady:
                chyby.append(f"{cesta.name}: chybí příklad / [en] example missing")
            for format_, text in priklady:
                over(format_, f"{cesta.name} (příklad / example)", text)


def kontrola():
    """Všechny kontroly; při chybě je vypíše a skončí s kódem 1.

    [en] All checks; on failure prints them and exits with code 1.
    """
    chyby = []
    schemata = {n: json.loads(c.read_text(encoding="utf-8")) for n, c in SCHEMATA.items()}
    radky = zkontroluj_historii(chyby)
    zkontroluj_verze_od(radky, schemata, chyby)
    zkontroluj_schemata(schemata, chyby)
    if chyby:
        for c in chyby:
            print(f"::error::{c}")
        sys.exit(1)
    pocet = sum(len(v) for v in VZORY.values())
    print(
        f"Formáty souborů v pořádku: 2 schémata, {pocet} vzorových souborů, příklady "
        f"ve 4 specifikacích, historie do verze {verze_programu()}. "
        f"/ [en] File formats OK."
    )


def main():
    """Vstupní bod příkazové řádky.

    [en] Command line entry point.
    """
    if sys.argv[1:] == ["kontrola"]:
        kontrola()
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main()
