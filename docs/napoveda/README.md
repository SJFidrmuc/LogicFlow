# Nápověda LogicFlow (zdroj)

Nápověda se píše v Markdownu a je **jediným zdrojem** pro nápovědu na webu i v programu:

- **Web:** při vydání ji workflow *Vydání* sestaví nástrojem [mdBook](https://rust-lang.github.io/mdBook/), připojí k vydání jako ZIP s HTML a (je-li zapnuté) zveřejní na GitHub Pages.
- **Program:** od verze 0.4 (etapa J2 plánu) se stejné soubory vloží do programu a zobrazí pod klávesou F1. Do té doby platí vestavěná nápověda v `crates/logicflow/src/help.rs` – při změně ji upravte také.

## Struktura

```
docs/napoveda/
├── cs/                     česká nápověda (výchozí)
│   ├── book.toml           nastavení mdBooku
│   ├── logicflow.css       barvy LogicFlow
│   └── src/
│       ├── SUMMARY.md      obsah = pořadí kapitol
│       ├── uvod.md         úvodní stránka
│       ├── zaciname.md …   kapitoly (jeden soubor = jedna kapitola)
│       └── obrazky/        snímky obrazovky
├── en/                     anglická nápověda – stejné názvy souborů jako cs
└── obvody/                 obvody .logicflow, ze kterých vznikají snímky
```

Názvy souborů kapitol jsou ve všech jazycích stejné (české), liší se jen obsah. Test `crates/logicflow/tests/napoveda.rs` (běží v `cargo test` i v CI) hlídá, že:

- česká a anglická nápověda mají stejné kapitoly ve stejném pořadí,
- každá kapitola existuje, začíná nadpisem `# …` a je uvedená v `SUMMARY.md`,
- každý obrázek, na který text odkazuje, je v repozitáři.

## Náhled

```sh
cargo install mdbook --locked          # jednou
mdbook serve docs/napoveda/cs --open   # průběžný náhled v prohlížeči
mdbook build docs/napoveda/cs          # výstup v target/napoveda/cs
```

## Jak psát

Pište tak, aby text šel zobrazit i v programu. Používejte jen tyto prvky:

| Prvek | Zápis | Použití |
| --- | --- | --- |
| nadpis kapitoly | `# Název` | jednou na začátku souboru |
| podnadpis | `## Název` | části kapitoly (třetí úroveň nepoužívat) |
| odstavec | prostý text | nejvýš 3 věty |
| odrážky | `- …` | souběžné položky |
| postup | `1. …` | kroky v daném pořadí |
| tabulka | `\| a \| b \|` | 2–3 sloupce: klávesa → akce, součástka → funkce |
| tip, upozornění | `> **Tip:** …` | jeden odstavec |
| zvýraznění | `**Shift+A**`, `**Soubor → Nový**` | klávesy, nabídky, tlačítka |
| kód, soubor | `` `logicflow.exe` `` | názvy souborů a příkazy |
| obrázek | `![popis](obrazky/nazev.png)` | samostatný řádek, popis = alternativní text |
| odkaz | `[text](kapitola.md)` | na jinou kapitolu nebo web |

Zásady:

- **Vykání** a krátké věty. Nabídky zapisujte se šipkou: **Zobrazit → Značky**.
- Popisujte jen to, co program skutečně umí ve verzi, se kterou se nápověda vydá. Připravované funkce označte „Připravuje se“.
- Názvy prvků rozhraní pište přesně jako v programu (texty z `crates/logicflow/src/i18n.rs`).
- Nová kapitola = nový soubor v `cs/src` i `en/src` a řádek v obou `SUMMARY.md`.

## Snímky obrazovky

1. Obvod pro snímek uložte do `docs/napoveda/obvody/` (nejlépe stejný název jako obrázek).
2. `cargo build --release` a `scripts/snimky-napovedy.sh` – snímky celého okna vzniknou v `target/snimky/`.
3. Výřez (a případné popisky) uložte jako PNG do `cs/src/obrazky/` (šířka do 1700 px, okna se zaoblenými rohy a stínem jako ostatní snímky).
4. Anglické snímky: `LANG_UI=en_GB.UTF-8 scripts/snimky-napovedy.sh`, ukládají se do `en/src/obrazky/`.
