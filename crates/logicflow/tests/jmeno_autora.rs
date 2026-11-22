//! Kontrola, že se jméno autora všude uvádí i s titulem („Mgr. Stanislav Jan Fidrmuc“)
//! – v kódu, nápovědě, knihovnách, licenci i dokumentaci.
//!
//! [en] Check that the author's name is given with the title everywhere
//! ("Mgr. Stanislav Jan Fidrmuc") – in the code, help, libraries, licence and documentation.

use std::fs;
use std::path::{Path, PathBuf};

/// Jméno autora bez titulu.
///
/// [en] The author's name without the title.
const NAME: &str = "Stanislav Jan Fidrmuc";
/// Jméno autora s titulem.
///
/// [en] The author's name with the title.
const FULL: &str = "Mgr. Stanislav Jan Fidrmuc";

/// Kořen repozitáře.
///
/// [en] Repository root.
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Textové soubory repozitáře (bez složek target a .git).
///
/// [en] Text files of the repository (without the target and .git folders).
fn text_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let path = e.path();
        let name = e.file_name().to_string_lossy().to_string();
        if path.is_dir() {
            if name != "target" && !name.starts_with('.') {
                text_files(&path, out);
            }
        } else if [
            "rs",
            "md",
            "toml",
            "lflib",
            "logicflow",
            "json",
            "yml",
            "py",
            "sh",
            "svg",
            "txt",
        ]
        .iter()
        .any(|ext| name.ends_with(&format!(".{ext}")))
            || name == "LICENSE"
        {
            out.push(path);
        }
    }
}

#[test]
fn author_name_has_title() {
    let mut files = Vec::new();
    text_files(&root(), &mut files);
    text_files(&root().join(".github"), &mut files);
    let mut bad = Vec::new();
    for f in files {
        // Tento test jméno bez titulu obsahuje záměrně.
        // [en] This test contains the name without the title on purpose.
        if f.ends_with("jmeno_autora.rs") {
            continue;
        }
        let Ok(text) = fs::read_to_string(&f) else {
            continue;
        };
        let without = text.matches(NAME).count();
        let with = text.matches(FULL).count();
        if without != with {
            bad.push(f.display().to_string());
        }
    }
    assert!(
        bad.is_empty(),
        "jméno autora bez titulu Mgr. v: {bad:?} / [en] author name without the title in"
    );
    let license = fs::read_to_string(root().join("LICENSE")).expect("LICENSE");
    assert!(license.contains(FULL), "LICENSE bez jména autora s titulem");
}
