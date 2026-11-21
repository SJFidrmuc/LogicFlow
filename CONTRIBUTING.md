# Jak se na LogicFlow pracuje

## Postup

1. Každá změna vzniká ve vlastní větvi (např. `m4-sbernice`) a do `main` se dostane přes pull request.
2. CI na každé větvi kontroluje formát a clippy, spouští testy na 6 platformách a sestavuje nápovědu.
3. Pull request se slučuje, až je CI zelené a kontrolní seznam v šabloně PR je splněný.
4. Vydání se dělá až po sloučení do `main`, s tagem ve tvaru `vX.Y.Z` (viz README → Vydání nové verze).

## Struktura kódu

Pro orientaci v kódu slouží diagram [docs/architektura.svg](docs/architektura.svg): aplikace (`crates/logicflow`) s hlavní smyčkou, editory a stavem projektu (`Project`) a jádro bez grafiky (`crates/logicflow-core`) s modelem obvodu (`Circuit`) a simulátorem (`Simulator`). Dole jsou popsané typické toky dat – úprava schématu, přepínač, běh času, kreslení a soubor.

Když změna přidá nový modul nebo změní tok dat (například soubor s nastavením, knihovny, sběrnice), upravte i diagram. Je to ručně psané SVG; náhled jako PNG: `rsvg-convert docs/architektura.svg -o /tmp/architektura.png`.

## Kdy je změna hotová

Změna je hotová, když jsou hotové **všechny** tyto části – ve stejném pull requestu:

- **kód** a k němu **testy** (jádro v `logicflow-core` má testy na simulaci a formát souboru),
- **texty rozhraní** česky i anglicky (`crates/logicflow/src/i18n.rs`),
- **nápověda v Markdownu** v `docs/napoveda/cs` i `docs/napoveda/en` – nová funkce má svůj popis, změněné chování opravený text, nové klávesy jsou v kapitole Klávesové zkratky,
- **snímky obrazovky** v nápovědě, pokud se změnil vzhled (`scripts/snimky-napovedy.sh`),
- **README** a plán vývoje u nových funkcí, případně diagram `docs/architektura.svg` při změně struktury.

Do verze 0.4 je nápověda ještě i přímo v kódu (`crates/logicflow/src/help.rs`). Od verze 0.4 (etapa J2) bude program zobrazovat přímo soubory z `docs/napoveda` a tento dvojí zápis odpadne.

Anglická nápověda může být při vývoji o krok pozadu, ale **kapitoly musí existovat** v obou jazycích – hlídá to test `crates/logicflow/tests/napoveda.rs`.

## Užitečné příkazy

```sh
cargo run                              # spuštění
cargo test --workspace                 # testy (včetně kontroly nápovědy)
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt
mdbook serve docs/napoveda/cs --open   # náhled nápovědy (cargo install mdbook --locked)
scripts/snimky-napovedy.sh             # snímky obrazovky pro nápovědu (Linux)
```

Podrobnosti k psaní nápovědy jsou v [docs/napoveda/README.md](docs/napoveda/README.md).
