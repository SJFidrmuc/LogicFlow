//! Kontrola nápovědy v Markdownu (docs/napoveda).
//!
//! Hlídá, aby česká a anglická nápověda měly stejné kapitoly ve stejném
//! pořadí, aby každá kapitola existovala a začínala nadpisem a aby všechny
//! obrázky, na které text odkazuje, byly v repozitáři.

use std::fs;
use std::path::{Path, PathBuf};

const LANGS: [&str; 2] = ["cs", "en"];

fn napoveda() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/napoveda")
}

fn src(lang: &str) -> PathBuf {
    napoveda().join(lang).join("src")
}

/// Soubory kapitol v pořadí podle SUMMARY.md.
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

#[test]
fn hledani_obrazku() {
    assert_eq!(
        images("text ![A](obrazky/a.png) a ![B](b.png)."),
        vec!["obrazky/a.png", "b.png"]
    );
}
