#!/usr/bin/env python3
"""Dvojjazyčné komentáře ve zdrojovém kódu (česky + anglicky).

[en] Bilingual source comments (Czech + English).

Pravidlo: každý blok komentářů na samostatných řádcích (`//`, `///`, `//!`),
který obsahuje text, má nejdřív českou část a pak anglickou část uvozenou
značkou `[en]`. Komentáře za kódem na stejném řádku smějí obsahovat jen
jazykově neutrální popisky (např. `// a`, `// 0–7`).

[en] Rule: every block of full-line comments (`//`, `///`, `//!`) containing
text has a Czech part first and then an English part introduced by the
`[en]` marker. Trailing comments after code may only hold language-neutral
labels (e.g. `// a`, `// 0–7`).

Použití / usage:
    python3 scripts/komentare.py kontrola        # kontrola pravidla (CI)
    python3 scripts/komentare.py en [složka]     # kopie zdrojů jen s anglickými komentáři
    python3 scripts/komentare.py cs [složka]     # kopie zdrojů jen s českými komentáři

Výchozí složka pro export je `target/zdroj-<jazyk>`.
[en] The default export folder is `target/zdroj-<language>`.
"""

import re
import shutil
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MARKER = "[en]"
PREFIXES = ("//!", "///", "//")
WORD = re.compile(r"[^\W\d_]{3,}")


def source_files():
    """Všechny zdrojové soubory Rustu v crates/.

    [en] All Rust source files under crates/.
    """
    return sorted((ROOT / "crates").glob("**/*.rs"))


def scan(text):
    """Rozdělí soubor na řádky a u každého určí, zda je celý komentářem.

    Vrací seznam dvojic (řádek, komentář), kde komentář je (odsazení, prefix,
    text) pro řádek tvořený jen komentářem, nebo None. Lexer přeskakuje
    řetězce (i víceřádkové a raw), znaky a blokové komentáře, aby `//` uvnitř
    řetězce nepovažoval za komentář. Vrací i seznam komentářů za kódem.

    [en] Splits the file into lines and decides for each whether it is a
    comment-only line. Returns a list of (line, comment) pairs, where comment
    is (indent, prefix, text) for comment-only lines, or None. The lexer skips
    strings (including multi-line and raw ones), chars and block comments so
    that `//` inside a string is not taken for a comment. Also returns the
    list of trailing comments after code.
    """
    lines = text.split("\n")
    result = []
    trailing = []
    state = None  # None | "str" | ("raw", hashes) | ("block", depth)
    for no, line in enumerate(lines, 1):
        i = 0
        n = len(line)
        code_seen = state is not None
        comment = None
        while i < n:
            c = line[i]
            if state == "str":
                if c == "\\":
                    i += 2
                    continue
                if c == '"':
                    state = None
                i += 1
                continue
            if isinstance(state, tuple) and state[0] == "raw":
                end = '"' + "#" * state[1]
                if line.startswith(end, i):
                    state = None
                    i += len(end)
                else:
                    i += 1
                continue
            if isinstance(state, tuple) and state[0] == "block":
                if line.startswith("*/", i):
                    depth = state[1] - 1
                    state = None if depth == 0 else ("block", depth)
                    i += 2
                elif line.startswith("/*", i):
                    state = ("block", state[1] + 1)
                    i += 2
                else:
                    i += 1
                continue
            # běžný kód / [en] plain code
            if line.startswith("//", i):
                body = line[i:]
                if not code_seen:
                    for p in PREFIXES:
                        if body.startswith(p):
                            comment = (line[:i], p, body[len(p):])
                            break
                else:
                    trailing.append((no, body[2:].strip()))
                break
            if line.startswith("/*", i):
                state = ("block", 1)
                code_seen = True
                i += 2
                continue
            m = re.match(r'b?r(#*)"', line[i:])
            if m and (i == 0 or not (line[i - 1].isalnum() or line[i - 1] == "_")):
                state = ("raw", len(m.group(1)))
                code_seen = True
                i += len(m.group(0))
                continue
            if c == '"':
                state = "str"
                code_seen = True
                i += 1
                continue
            if c == "'":
                # Znak 'x', '\n', '\u{..}' – jinak jde o lifetime.
                # [en] A char literal 'x', '\n', '\u{..}' – otherwise a lifetime.
                m = re.match(r"'(\\u\{[0-9a-fA-F]+\}|\\.|[^\\'])'", line[i:])
                code_seen = True
                i += len(m.group(0)) if m else 1
                continue
            if not c.isspace():
                code_seen = True
            i += 1
        result.append((line, comment))
    return result, trailing


def blocks(scanned):
    """Seskupí po sobě jdoucí komentářové řádky se stejným prefixem a odsazením.

    Vrací seznam (index prvního řádku, index za posledním řádkem).

    [en] Groups consecutive comment lines with the same prefix and indent.
    Returns a list of (index of the first line, index past the last line).
    """
    out = []
    i = 0
    while i < len(scanned):
        c = scanned[i][1]
        if c is None:
            i += 1
            continue
        j = i + 1
        while (
            j < len(scanned)
            and scanned[j][1] is not None
            and scanned[j][1][:2] == c[:2]
        ):
            j += 1
        out.append((i, j))
        i = j
    return out


def check():
    """Ověří pravidlo ve všech souborech; vrací počet chyb.

    [en] Checks the rule in all files; returns the number of errors.
    """
    errors = 0
    for path in source_files():
        rel = path.relative_to(ROOT)
        scanned, trailing = scan(path.read_text(encoding="utf-8"))
        for start, end in blocks(scanned):
            texts = [scanned[k][1][2] for k in range(start, end)]
            joined = "\n".join(texts)
            if not WORD.search(joined):
                continue
            if MARKER not in joined:
                print(f"{rel}:{start + 1}: komentář bez anglické části [en]")
                errors += 1
                continue
            cs, en = joined.split(MARKER, 1)
            if not WORD.search(cs) or not WORD.search(en):
                print(f"{rel}:{start + 1}: komentář musí mít českou i anglickou část")
                errors += 1
        for no, body in trailing:
            if WORD.search(body):
                print(
                    f"{rel}:{no}: komentář za kódem smí obsahovat jen neutrální "
                    f"popisek – přesuňte ho na samostatný řádek"
                )
                errors += 1
    return errors


def filter_language(text, lang):
    """Vrátí text souboru jen s komentáři v jazyce `lang` ("cs" nebo "en").

    [en] Returns the file text with comments in language `lang` only.
    """
    scanned, _ = scan(text)
    keep = [True] * len(scanned)
    replace = {}
    for start, end in blocks(scanned):
        texts = [scanned[k][1][2] for k in range(start, end)]
        marker_at = next((k for k, t in enumerate(texts) if MARKER in t), None)
        if marker_at is None:
            continue
        line_idx = start + marker_at
        indent, prefix, body = scanned[line_idx][1]
        before, after = body.split(MARKER, 1)
        if lang == "cs":
            for k in range(line_idx, end):
                keep[k] = False
            if before.strip():
                keep[line_idx] = True
                replace[line_idx] = indent + prefix + before.rstrip()
            # Prázdný oddělovací řádek před [en] odstraníme.
            # [en] Drop the empty separator line before [en].
            k = line_idx - 1
            while k >= start and keep[k] and not scanned[k][1][2].strip():
                keep[k] = False
                k -= 1
        else:
            for k in range(start, line_idx):
                keep[k] = False
            text_en = after.lstrip()
            if text_en:
                replace[line_idx] = indent + prefix + " " + text_en
            else:
                # Značka na samostatném řádku: vynechá se i s prázdnými řádky za ní.
                # [en] Marker on its own line: dropped together with empty lines after it.
                keep[line_idx] = False
                k = line_idx + 1
                while k < end and not scanned[k][1][2].strip():
                    keep[k] = False
                    k += 1
    out = []
    for k, (line, _) in enumerate(scanned):
        if keep[k]:
            out.append(replace.get(k, line))
    return "\n".join(out)


def export(lang, target):
    """Zkopíruje crates/ do `target` s komentáři jen v jednom jazyce.

    [en] Copies crates/ into `target` with comments in one language only.
    """
    dest = Path(target)
    if dest.exists():
        shutil.rmtree(dest)
    shutil.copytree(ROOT / "crates", dest / "crates", ignore=shutil.ignore_patterns("target"))
    for name in ("Cargo.toml", "Cargo.lock", "LICENSE", "README.md"):
        shutil.copy2(ROOT / name, dest / name)
    for path in source_files():
        rel = path.relative_to(ROOT)
        (dest / rel).write_text(
            filter_language(path.read_text(encoding="utf-8"), lang), encoding="utf-8"
        )
    print(f"Zdrojový kód s komentáři „{lang}“: {dest}")


def main():
    if len(sys.argv) < 2 or sys.argv[1] not in ("kontrola", "cs", "en"):
        print(__doc__)
        return 2
    if sys.argv[1] == "kontrola":
        errors = check()
        if errors:
            print(f"\nChyb: {errors}. Pravidla: CONTRIBUTING.md → Komentáře v kódu.")
            return 1
        print("Komentáře: v pořádku (česky i anglicky).")
        return 0
    lang = sys.argv[1]
    target = sys.argv[2] if len(sys.argv) > 2 else ROOT / "target" / f"zdroj-{lang}"
    export(lang, target)
    return 0


if __name__ == "__main__":
    sys.exit(main())
