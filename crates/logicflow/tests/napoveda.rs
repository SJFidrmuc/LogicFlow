//! Kontrola nápovědy v Markdownu (docs/napoveda).
//!
//! Hlídá, aby česká a anglická nápověda měly stejné kapitoly ve stejném
//! pořadí, aby každá kapitola existovala a začínala nadpisem a aby všechny
//! obrázky, na které text odkazuje, byly v repozitáři.
//!
//! [en] Check of the Markdown help (docs/napoveda).
//!
//! Makes sure the Czech and English help have the same chapters in the same
//! order, that every chapter exists and starts with a heading and that all
//! images referenced by the text are in the repository.

use std::fs;
use std::path::{Path, PathBuf};

/// Jazyky nápovědy.
///
/// [en] Help languages.
const LANGS: [&str; 2] = ["cs", "en"];

/// Složka docs/napoveda.
///
/// [en] The docs/napoveda folder.
fn napoveda() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/napoveda")
}

/// Složka se zdroji nápovědy v daném jazyce.
///
/// [en] Folder with the help sources in the given language.
fn src(lang: &str) -> PathBuf {
    napoveda().join(lang).join("src")
}

/// Soubory kapitol v pořadí podle SUMMARY.md.
///
/// [en] Chapter files in the order of SUMMARY.md.
fn chapters(lang: &str) -> Vec<String> {
    let summary = fs::read_to_string(src(lang).join("SUMMARY.md"))
        .unwrap_or_else(|e| panic!("{lang}: chybí SUMMARY.md: {e}"));
    summary
        .lines()
        .filter_map(|line| {
            let start = line.find("](")? + 2;
            let end = line[start..].find(')')? + start;
            Some(line[start..end].to_owned())
        })
        .collect()
}

/// Cesty obrázků z Markdownu: `![popis](cesta)`.
///
/// [en] Image paths from Markdown: `![caption](path)`.
fn images(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(i) = rest.find("![") {
        rest = &rest[i + 2..];
        let Some(j) = rest.find("](") else { break };
        let after = &rest[j + 2..];
        let Some(k) = after.find(')') else { break };
        out.push(after[..k].to_owned());
        rest = &after[k..];
    }
    out
}

/// Česká a anglická nápověda mají stejné kapitoly ve stejném pořadí.
///
/// [en] The Czech and English help have the same chapters in the same order.
#[test]
fn jazyky_maji_stejne_kapitoly() {
    let cs = chapters("cs");
    assert!(!cs.is_empty(), "cs: SUMMARY.md neobsahuje žádné kapitoly");
    for lang in LANGS {
        assert_eq!(
            chapters(lang),
            cs,
            "{lang}: kapitoly v SUMMARY.md se liší od české nápovědy (stejné soubory ve stejném pořadí)"
        );
    }
}

/// Každá kapitola existuje, začíná nadpisem a je v obsahu.
///
/// [en] Every chapter exists, starts with a heading and is in the table of contents.
#[test]
fn kapitoly_existuji_a_zacinaji_nadpisem() {
    for lang in LANGS {
        let listed = chapters(lang);
        for ch in &listed {
            let path = src(lang).join(ch);
            let text = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("{lang}: kapitola {ch} chybí: {e}"));
            assert!(
                text.starts_with("# "),
                "{lang}/{ch}: kapitola musí začínat nadpisem první úrovně (# …)"
            );
        }
        // Každý soubor .md ve složce src musí být v obsahu.
        // [en] Every .md file in the src folder must be in the table of contents.
        for entry in fs::read_dir(src(lang)).unwrap() {
            let name = entry.unwrap().file_name().into_string().unwrap();
            if name.ends_with(".md") && name != "SUMMARY.md" {
                assert!(
                    listed.contains(&name),
                    "{lang}/{name}: kapitola není uvedená v SUMMARY.md"
                );
            }
        }
    }
}

/// Všechny obrázky, na které nápověda odkazuje, existují.
///
/// [en] All images referenced by the help exist.
#[test]
fn obrazky_existuji() {
    for lang in LANGS {
        for ch in chapters(lang) {
            let text = fs::read_to_string(src(lang).join(&ch)).unwrap();
            for img in images(&text) {
                if img.starts_with("http") {
                    continue;
                }
                assert!(
                    src(lang).join(&img).is_file(),
                    "{lang}/{ch}: obrázek {img} neexistuje"
                );
            }
        }
    }
}

/// Hledání obrázků v textu funguje.
///
/// [en] Finding images in the text works.
#[test]
fn hledani_obrazku() {
    assert_eq!(
        images("text ![A](obrazky/a.png) a ![B](b.png)."),
        vec!["obrazky/a.png", "b.png"]
    );
}
