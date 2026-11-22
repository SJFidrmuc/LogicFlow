#!/usr/bin/env python3
"""Archiv verzí LogicFlow: tabulka vydání × platforma s odkazy ke stažení.

Data bere z GitHub API (seznam vydání), binárky zůstávají ve vydáních na GitHubu –
do repozitáře se neukládají. Výstup:

  html VYDANI.json SLOZKA   stránka SLOZKA/index.html (česky i anglicky) pro web
  md VYDANI.json            tabulka v Markdownu (do popisu průběžného sestavení)
  test                      kontrola na ukázkových datech (spouští CI)

VYDANI.json vznikne příkazem:
  gh api repos/SJFidrmuc/LogicFlow/releases --paginate --slurp > vydani.json

[en] LogicFlow version archive: a table of releases × platforms with download links.

It takes the data from the GitHub API (list of releases); the binaries stay in the
releases on GitHub – they are not stored in the repository. Output:

  html RELEASES.json FOLDER  page FOLDER/index.html (Czech and English) for the web
  md RELEASES.json           a Markdown table (for the rolling build description)
  test                       check on sample data (run by CI)
"""

import html
import json
import os
import sys
import tempfile

# Platformy: přípona názvu souboru ve vydání, český a anglický název.
# [en] Platforms: file name suffix in the release, Czech and English name.
PLATFORMS = [
    ("-windows-x64.zip", "Windows x64", "Windows x64"),
    ("-windows-arm64.zip", "Windows ARM64", "Windows ARM64"),
    ("-macos-arm64.tar.gz", "macOS (Apple)", "macOS (Apple)"),
    ("-macos-x64.tar.gz", "macOS (Intel)", "macOS (Intel)"),
    ("-linux-x64.tar.gz", "Linux x64", "Linux x64"),
    ("-raspberry-pi-arm64.tar.gz", "Raspberry Pi", "Raspberry Pi"),
    ("-napoveda-html.zip", "Nápověda (HTML)", "Help (HTML)"),
]

# Tag průběžného sestavení z větve main.
# [en] Tag of the rolling build from the main branch.
ROLLING = "prubezne"


def load(path):
    """Načte seznam vydání; `gh api --paginate --slurp` dává seznam stránek.

    [en] Loads the list of releases; `gh api --paginate --slurp` gives a list of pages.
    """
    data = json.load(open(path, encoding="utf-8"))
    if data and isinstance(data[0], list):
        data = [r for page in data for r in page]
    return [r for r in data if not r.get("draft")]


def rows(releases):
    """Řádky tabulky: průběžné sestavení nahoře, pak vydání od nejnovějšího.

    [en] Table rows: the rolling build at the top, then releases from the newest.
    """
    rolling = [r for r in releases if r.get("tag_name") == ROLLING]
    others = [r for r in releases if r.get("tag_name") != ROLLING]
    others.sort(key=lambda r: r.get("published_at") or "", reverse=True)
    out = []
    for r in rolling + others:
        links = {}
        for a in r.get("assets", []):
            for suffix, _, _ in PLATFORMS:
                if a["name"].endswith(suffix):
                    links[suffix] = (a["browser_download_url"], a.get("size", 0))
        out.append(
            {
                "tag": r["tag_name"],
                "name": r.get("name") or r["tag_name"],
                "date": (r.get("published_at") or "")[:10],
                "url": r.get("html_url", ""),
                "rolling": r["tag_name"] == ROLLING,
                "prerelease": bool(r.get("prerelease")),
                "links": links,
            }
        )
    return out


def size_text(size):
    """Velikost souboru v MB.

    [en] File size in MB.
    """
    return f"{size / 1_048_576:.1f} MB"


def markdown(releases):
    """Tabulka v Markdownu (česky; názvy platforem jsou stejné).

    [en] A Markdown table (Czech; platform names are the same).
    """
    head = ["Verze", "Datum"] + [cs for _, cs, _ in PLATFORMS]
    lines = ["| " + " | ".join(head) + " |", "|" + " --- |" * len(head)]
    for row in rows(releases):
        name = "průběžné (main)" if row["rolling"] else row["tag"]
        cells = [f"[{name}]({row['url']})", row["date"]]
        for suffix, _, _ in PLATFORMS:
            link = row["links"].get(suffix)
            cells.append(f"[stáhnout]({link[0]})" if link else "–")
        lines.append("| " + " | ".join(cells) + " |")
    return "\n".join(lines) + "\n"


def page(releases):
    """Samostatná stránka HTML (česky i anglicky, přepínač jazyka bez skriptu).

    [en] A self-contained HTML page (Czech and English, a language switch without a script).
    """
    def table(lang):
        en = lang == "en"
        head = ["Version" if en else "Verze", "Date" if en else "Datum"]
        head += [(e if en else c) for _, c, e in PLATFORMS]
        out = ["<table><thead><tr>"]
        out += [f"<th>{html.escape(h)}</th>" for h in head]
        out.append("</tr></thead><tbody>")
        for row in rows(releases):
            if row["rolling"]:
                label = "Rolling build (main)" if en else "Průběžné sestavení (main)"
                note = (
                    "latest state of development, not an official release"
                    if en
                    else "aktuální stav vývoje, není to oficiální vydání"
                )
                name = f'<a href="{html.escape(row["url"])}">{label}</a><br><small>{note}</small>'
                cls = ' class="rolling"'
            else:
                name = f'<a href="{html.escape(row["url"])}">{html.escape(row["tag"])}</a>'
                if row["prerelease"]:
                    name += "<br><small>pre-release</small>" if en else "<br><small>předběžné</small>"
                cls = ""
            out.append(f"<tr{cls}><td>{name}</td><td>{html.escape(row['date'])}</td>")
            for suffix, _, _ in PLATFORMS:
                link = row["links"].get(suffix)
                if link:
                    word = "download" if en else "stáhnout"
                    out.append(
                        f'<td><a href="{html.escape(link[0])}">{word}</a>'
                        f"<br><small>{size_text(link[1])}</small></td>"
                    )
                else:
                    out.append("<td>–</td>")
            out.append("</tr>")
        out.append("</tbody></table>")
        return "".join(out)

    return f"""<!DOCTYPE html>
<html lang="cs">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>LogicFlow – archiv verzí / version archive</title>
<style>
body {{ font-family: "Fira Sans", system-ui, sans-serif; margin: 0; color: #1c1c1c; background: #fafafa; }}
header {{ background: #1c1c1c; color: #fff; padding: 18px 24px; }}
header b {{ color: #7ee07e; }}
main {{ padding: 20px 24px; max-width: 1200px; }}
.lang {{ float: right; }}
.lang a {{ color: #7ee07e; margin-left: 12px; }}
table {{ border-collapse: collapse; width: 100%; background: #fff; font-size: 14px; }}
th, td {{ border: 1px solid #ddd; padding: 6px 8px; text-align: left; vertical-align: top; }}
th {{ background: #237a35; color: #fff; }}
tr.rolling td {{ background: #f1f8f2; }}
a {{ color: #237a35; }}
small {{ color: #666; }}
#en {{ display: none; }}
#en:target {{ display: block; }}
#en:target ~ #cs {{ display: none; }}
</style>
</head>
<body>
<header><span class="lang"><a href="#cs">Česky</a><a href="#en">English</a></span>Logic<b>Flow</b> – archiv verzí</header>
<main>
<section id="en">
<h1>Version archive</h1>
<p>All LogicFlow releases for every platform. Each package contains the program, README and LICENSE; the release page has the release notes. The rolling build is the latest state of development from the main branch – for trying out, not for teaching.</p>
{table("en")}
</section>
<section id="cs">
<h1>Archiv verzí</h1>
<p>Všechna vydání LogicFlow pro všechny platformy. Každý balíček obsahuje program, README a licenci; na stránce vydání jsou poznámky k vydání. Průběžné sestavení je aktuální stav vývoje z větve main – na vyzkoušení, ne do výuky.</p>
{table("cs")}
</section>
</main>
</body>
</html>
"""


def self_test():
    """Kontrola na ukázkových datech.

    [en] Check on sample data.
    """
    sample = [
        [
            {
                "tag_name": "v0.2.0",
                "name": "LogicFlow v0.2.0",
                "published_at": "2026-11-19T12:04:13Z",
                "html_url": "https://example/v0.2.0",
                "assets": [
                    {"name": "LogicFlow-0.2.0-windows-x64.zip", "browser_download_url": "https://example/w.zip", "size": 4_000_000},
                ],
            },
            {
                "tag_name": "v0.3.1",
                "published_at": "2026-11-21T09:10:02Z",
                "html_url": "https://example/v0.3.1",
                "assets": [
                    {"name": "LogicFlow-0.3.1-linux-x64.tar.gz", "browser_download_url": "https://example/l.tgz", "size": 6_000_000},
                    {"name": "LogicFlow-0.3.1-napoveda-html.zip", "browser_download_url": "https://example/n.zip", "size": 2_000_000},
                ],
            },
            {
                "tag_name": ROLLING,
                "prerelease": True,
                "published_at": "2026-11-22T10:00:00Z",
                "html_url": "https://example/prubezne",
                "assets": [
                    {"name": "LogicFlow-prubezne-raspberry-pi-arm64.tar.gz", "browser_download_url": "https://example/r.tgz", "size": 5_000_000},
                ],
            },
            {"tag_name": "v9.9.9", "draft": True, "assets": []},
        ]
    ]
    with tempfile.TemporaryDirectory() as tmp:
        path = os.path.join(tmp, "vydani.json")
        json.dump(sample, open(path, "w", encoding="utf-8"))
        releases = load(path)
        assert [r["tag"] for r in rows(releases)] == [ROLLING, "v0.3.1", "v0.2.0"], "pořadí"
        md = markdown(releases)
        assert "[stáhnout](https://example/l.tgz)" in md and "v9.9.9" not in md, md
        text = page(releases)
        assert "Průběžné sestavení" in text and "Rolling build" in text
        assert "https://example/r.tgz" in text and "5.7 MB" in text
    print("Archiv verzí: v pořádku.")


def main():
    """Zpracuje příkazovou řádku.

    [en] Handles the command line.
    """
    args = sys.argv[1:]
    if args[:1] == ["test"]:
        self_test()
    elif args[:1] == ["md"] and len(args) == 2:
        sys.stdout.write(markdown(load(args[1])))
    elif args[:1] == ["html"] and len(args) == 3:
        os.makedirs(args[2], exist_ok=True)
        with open(os.path.join(args[2], "index.html"), "w", encoding="utf-8") as f:
            f.write(page(load(args[1])))
        print(f"Zapsán {os.path.join(args[2], 'index.html')}")
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main()
