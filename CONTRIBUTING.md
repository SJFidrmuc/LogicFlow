# Jak se na LogicFlow pracuje

## Postup

1. Každá změna vzniká ve vlastní větvi (např. `m4-sbernice`) a do `main` se dostane přes pull request.
2. CI na každé větvi kontroluje formát a clippy, spouští testy na 6 platformách a sestavuje nápovědu.
3. Pull request se slučuje, až je CI zelené a kontrolní seznam v šabloně PR je splněný.
4. Vydání se dělá až po sloučení do `main`, s tagem ve tvaru `vX.Y.Z` (viz README → Vydání nové verze).

## Struktura kódu

Pro orientaci v kódu slouží diagram [docs/architektura.svg](docs/architektura.svg): aplikace (`crates/logicflow`) s hlavní smyčkou, editory a stavem projektu (`Project`) a jádro bez grafiky (`crates/logicflow-core`) s modelem obvodu (`Circuit`) a simulátorem (`Simulator`). Dole jsou popsané typické toky dat – úprava schématu, přepínač, běh času, kreslení a soubor.

Když změna přidá nový modul nebo změní tok dat (například soubor s nastavením, knihovny, sběrnice), upravte i diagram. Je to ručně psané SVG; náhled jako PNG: `rsvg-convert docs/architektura.svg -o /tmp/architektura.png`.

## Komentáře v kódu

Kód je komentovaný **česky i anglicky**, aby šla jednoduše vytvořit verze s komentáři jen v jednom jazyce – podobně jako jazykové mutace programu.

- Každá funkce, struktura, výčet a konstanta má dokumentační komentář (`///`), každý soubor popis modulu (`//!`). Složitější místa uvnitř funkcí mají běžný komentář (`//`), který vysvětluje **proč**, ne jen co.
- V každém komentáři je nejdřív česká část, pak anglická uvozená značkou `[en]`. U dokumentačních komentářů je mezi nimi prázdný řádek, aby je rustdoc zobrazil jako dva odstavce:

  ```rust
  /// Přepojí vodič k posunutému vývodu.
  ///
  /// [en] Reroutes the wire to the moved pin.
  fn reroute(/* … */) {
      // Pevný konec je roh lomeného vodiče.
      // [en] The fixed end is the corner of a bent wire.
  }
  ```

- Začíná-li anglická část nadpisem, tabulkou nebo seznamem, stojí `[en]` na samostatném řádku.
- Komentář za kódem na stejném řádku smí obsahovat jen jazykově neutrální popisek (`// a`, `// 0–7`); delší komentář patří na samostatný řádek nad kód.
- Při změně komentáře se upravují **obě** jazykové části.
- Jádro (`logicflow-core`) má zapnuté lints `missing_docs` a `clippy::missing_docs_in_private_items` – clippy v CI selže, když nějaké položce komentář chybí.

Nástroj `scripts/komentare.py` (Python 3, bez dalších knihoven):

```sh
python3 scripts/komentare.py kontrola   # má každý komentář obě části? (spouští i CI)
python3 scripts/komentare.py en         # kopie zdrojů jen s anglickými komentáři → target/zdroj-en
python3 scripts/komentare.py cs         # kopie zdrojů jen s českými komentáři → target/zdroj-cs
```

Exportovaná kopie je samostatný projekt, který jde přeložit (`cargo build`); slouží ke čtení nebo předání, vyvíjí se vždy ve dvojjazyčném zdroji.

## Přenositelnost (budoucí webová verze)

Kolem verze 1.0 vznikne webová verze (WebAssembly v prohlížeči, instalovatelná na plochu) pro iPad, iPhone, Android a Chromebooky – viz plán vývoje. Aby to pak nebyl velký zásah, platí už teď:

- **Jádro (`logicflow-core`) nepracuje se systémem:** žádné soubory, vlákna, proměnné prostředí ani systémový čas. Dostává a vrací text (JSON) a čas simulace mu předává aplikace. Hlídá to clippy podle `crates/logicflow-core/clippy.toml` a úloha CI **Jádro pro WebAssembly**, která jádro pro WebAssembly překládá.
- **Čas v aplikaci přes `web_time`** (`web_time::Instant`, `web_time::SystemTime`), ne `std::time` – `std::time::Instant::now()` ve WebAssembly spadne. V nativním programu je to totéž; kontrolu má úloha Formát a clippy.
- **Práce se soubory jen na vyhrazených místech:** `filedialog.rs` (vlastní okno pro soubory), `Project::load` / `Project::save` (tenké obaly kolem `Project::from_text` / `Project::to_text`), `main.rs` (soubor z příkazové řádky) a později soubor s nastavením a knihovny. Ostatní kód pracuje s textem nebo s `Project`, ne s cestami na disku. Webová verze tato místa nahradí nahráním / stažením souboru a úložištěm prohlížeče.
- **Nic neblokuje hlavní smyčku:** žádné `thread::sleep` ani čekání na síť v `update`. Dlouhé akce (kontrola aktualizací, stahování knihovny) na pozadí s výsledkem v dalším snímku; ve webové verzi pak asynchronně.
- **Každý příkaz jde vyvolat i bez klávesnice** – z nabídky, kontextové nabídky nebo panelu nástrojů (dotyk). Příkazy jsou na jednom místě a nabídky, klávesy i panel je jen volají.
- **Žádné pevné předpoklady o okně:** velikost okna, myš s pravým tlačítkem a najetí myší (tooltip) nemusí na tabletu existovat – důležitá informace nesmí být jen v tooltipu.
- **Nové závislosti** ověřte, že podporují `wasm32-unknown-unknown` (nebo je lze pro web vypnout přes `#[cfg(not(target_arch = "wasm32"))]`).

## Kdy je změna hotová

Změna je hotová, když jsou hotové **všechny** tyto části – ve stejném pull requestu:

- **kód** a k němu **testy** (jádro v `logicflow-core` má testy na simulaci a formát souboru),
- **komentáře** česky i anglicky u nových funkcí a struktur (viz Komentáře v kódu),
- **texty rozhraní** česky i anglicky (`crates/logicflow/src/i18n.rs`),
- **nápověda v Markdownu** v `docs/napoveda/cs` i `docs/napoveda/en` – nová funkce má svůj popis, změněné chování opravený text, nové klávesy jsou v kapitole Klávesové zkratky,
- **snímky obrazovky** v nápovědě, pokud se změnil vzhled (`scripts/snimky-napovedy.sh`),
- **přenositelnost** – jádro bez souborů a systémového času, čas v aplikaci přes `web_time` (viz Přenositelnost),
- **README** a plán vývoje u nových funkcí, případně diagram `docs/architektura.svg` při změně struktury.

Do verze 0.4 je nápověda ještě i přímo v kódu (`crates/logicflow/src/help.rs`). Od verze 0.4 (etapa J2) bude program zobrazovat přímo soubory z `docs/napoveda` a tento dvojí zápis odpadne.

Anglická nápověda může být při vývoji o krok pozadu, ale **kapitoly musí existovat** v obou jazycích – hlídá to test `crates/logicflow/tests/napoveda.rs`.

## Užitečné příkazy

```sh
cargo run                              # spuštění
cargo test --workspace                 # testy (včetně kontroly nápovědy)
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt
python3 scripts/komentare.py kontrola  # komentáře česky i anglicky
mdbook serve docs/napoveda/cs --open   # náhled nápovědy (cargo install mdbook --locked)
scripts/snimky-napovedy.sh             # snímky obrazovky pro nápovědu (Linux)
```

Podrobnosti k psaní nápovědy jsou v [docs/napoveda/README.md](docs/napoveda/README.md).
