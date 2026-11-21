## Co se mění

<!-- Stručně: co a proč. -->

## Kontrola

- [ ] `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings` a `cargo test --workspace` prošly
- [ ] Nové funkce, struktury a složitější místa mají komentář **česky i anglicky** (`python3 scripts/komentare.py kontrola`)
- [ ] Jádro bez souborů a systémového času, čas v aplikaci přes `web_time` (CONTRIBUTING.md → Přenositelnost)
- [ ] **Nápověda** (`docs/napoveda/cs` a `docs/napoveda/en`) odpovídá změně – nebo změna rozhraní ani chování nemění
- [ ] Do verze 0.4: změna je i ve vestavěné nápovědě `crates/logicflow/src/help.rs`
- [ ] Nové nebo změněné texty rozhraní jsou česky i anglicky (`i18n.rs`)
- [ ] Snímky obrazovky v nápovědě jsou aktuální (`scripts/snimky-napovedy.sh`), pokud se změnil vzhled
- [ ] Přehled změn: řádek v sekci Nevydáno v `CHANGELOG.md` i `CHANGELOG.en.md` (u změn viditelných pro uživatele)
- [ ] README / plán vývoje upraveny, pokud jde o novou funkci
