# Jak se na LogicFlow pracuje

## Postup

1. Každá změna vzniká ve vlastní větvi (např. `m4-sbernice`) a do `main` se dostane přes pull request.
2. CI na každé větvi kontroluje formát a clippy, spouští testy na 6 platformách a sestavuje nápovědu.
3. Pull request se slučuje, až je CI zelené a kontrolní seznam v šabloně PR je splněný.
4. Vydání se dělá až po sloučení do `main`, s tagem ve tvaru `vX.Y.Z` (viz Vydání nové verze níže).

## Struktura kódu

Pro orientaci v kódu slouží diagram [docs/architektura.svg](docs/architektura.svg): aplikace (`crates/logicflow`) s hlavní smyčkou, editory a stavem projektu (`Project`) a jádro bez grafiky (`crates/logicflow-core`) s modelem obvodu (`Circuit`) a simulátorem (`Simulator`). Dole jsou popsané typické toky dat – úprava schématu, přepínač, běh času, kreslení a soubor.

Když změna přidá nový modul nebo změní tok dat (například soubor s nastavením, knihovny, sběrnice), upravte i diagram. Je to ručně psané SVG; náhled jako PNG: `rsvg-convert docs/architektura.svg -o /tmp/architektura.png`.

## Formáty souborů

Soubory, které program čte a zapisuje, popisuje závazně složka [docs/formaty](docs/formaty/README.md): projekt `.logicflow` (i text ve schránce) a knihovna prvků `.lflib`, každá specifikace česky i anglicky a s JSON Schema. U každé součástky, hodnoty a pole je sloupec **Od verze** – první verze programu, která je zná; z toho se pozná, kterou verzí soubor otevřete.

Při změně, která formát dotkne, se ve stejném pull requestu upraví i dokumentace formátu:

- **Nová součástka, blok, druh hradla či klopného obvodu, nová hodnota nebo pole:** řádek ve specifikaci česky i anglicky (`logicflow.md` + `logicflow.en.md`, u knihovny `lflib.md` + `lflib.en.md`) s Od verze = připravovaná verze, položka v JSON Schema s anotací `x-since`, zápis do řádku připravované verze v historii (`docs/formaty/README.md`, označený „nevydáno“) a do testu `crates/logicflow-core/tests/formaty.rs` (funkce `*_since`, u součástky i `all_kinds`). Nový druh součástky bez zápisu v testu se nepřeloží.
- **Nové pole** musí mít výchozí hodnotu (`#[serde(default)]`), aby se dál otevřely soubory ze starších verzí.
- **Nekompatibilní změna** (přejmenování nebo odebrání klíče či hodnoty, jiný význam existující hodnoty) jen se zvýšením `FORMAT` v kódu, popisem v historii a načítáním starší verze formátu. Přednost má vždy kompatibilní řešení.
- **Vydání verze:** u řádku verze v historii formátů odstranit „nevydáno“ a v záhlaví specifikací a v přehledu formátů upravit stav dokumentace na novou verzi (viz Vydání nové verze). Vyjde-li verze pod jiným číslem, než bylo v plánu, opravit i hodnoty Od verze.

Kontrolují to test `formaty.rs` (shoda specifikací a schémat s kódem, příklady se načtou a správně počítají) a `python3 scripts/formaty.py kontrola` v CI (schémata, vzorové soubory, historie proti přehledu změn a verzi v `Cargo.toml`; potřebuje `pip install jsonschema`).

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
- **přehled změn** – řádek v sekci Nevydáno v [CHANGELOG.md](CHANGELOG.md) i [CHANGELOG.en.md](CHANGELOG.en.md), pokud změna mění něco, čeho si všimne uživatel (nová funkce, oprava, změna chování), nebo něco důležitého pro vývoj,
- **README** a plán vývoje u nových funkcí, případně diagram `docs/architektura.svg` při změně struktury,
- **dokumentace formátů** v `docs/formaty`, pokud změna přidá nebo změní součástku, pole nebo hodnotu v souboru `.logicflow` či `.lflib` (viz Formáty souborů).

Do verze 0.4 je nápověda ještě i přímo v kódu (`crates/logicflow/src/help.rs`). Od verze 0.4 (etapa J2) bude program zobrazovat přímo soubory z `docs/napoveda` a tento dvojí zápis odpadne.

Anglická nápověda může být při vývoji o krok pozadu, ale **kapitoly musí existovat** v obou jazycích – hlídá to test `crates/logicflow/tests/napoveda.rs`.

## Přehled změn a vydání nové verze

### Přehled změn

[CHANGELOG.md](CHANGELOG.md) (česky) a [CHANGELOG.en.md](CHANGELOG.en.md) (anglicky) popisují, co přinesla která verze a co se plánuje dál. Oba soubory mají stejnou stavbu:

- `## [Nevydáno]` / `## [Unreleased]` – změny, které vyjdou v příští verzi, rozdělené na **Přidáno / Added**, **Změněno / Changed**, **Opraveno / Fixed** (případně **Odstraněno / Removed**);
- `## [X.Y.Z] – RRRR-MM-DD` – vydané verze, nejnovější nahoře;
- `## Plán dalších verzí` / `## Roadmap` – stručný plán (podrobný plán vede autor projektu);
- na konci odkazy `[X.Y.Z]: …` na vydání nebo porovnání verzí.

Píše se pro uživatele (učitele, žáky): co se změnilo a k čemu to je, ne jak je to naprogramované. Jeden řádek = jedna změna. Kontrolu (stejné verze a data v obou jazycích, sekce Nevydáno, popis aktuální verze z `Cargo.toml`, odkazy) dělá `python3 scripts/zmeny.py kontrola` v CI.

### Vydání nové verze

1. **Pull request „Verze X.Y.Z“:**
   - zvýšit `version` v `Cargo.toml` (`[workspace.package]`) a stav verze v README;
   - v obou přehledech změn přejmenovat `[Nevydáno]` / `[Unreleased]` na `[X.Y.Z] – dnešní datum` a nad ni vložit novou prázdnou sekci Nevydáno / Unreleased (může obsahovat jen nadpis a prázdný řádek – kontrola prázdnou sekci Nevydáno připouští);
   - upravit odkazy na konci: `[Nevydáno]: …/compare/vX.Y.Z...HEAD` a nový `[X.Y.Z]: …/releases/tag/vX.Y.Z`;
   - upravit **Plán dalších verzí** (vydané body odebrat, posunout plán) a tabulku Plán v README;
   - v `docs/formaty`: u verze v historii formátů odstranit „nevydáno“ / „unreleased“ a stav dokumentace v `README.md` a v záhlaví čtyř specifikací přepsat na novou verzi (`scripts/formaty.py kontrola` jinak selže);
   - CI musí být zelené (`scripts/zmeny.py kontrola` hlídá, že verze z `Cargo.toml` je v přehledu popsaná).
2. Pull request sloučit do `main`.
3. Na GitHubu *Releases → Draft a new release* → **nový** tag `vX.Y.Z` (*Create new tag on publish*), cíl *main*. Tag se nikdy nevytváří předem – existující tag by se použil znovu i se starou revizí. Popis vydání lze nechat **prázdný**: workflow ho doplní z přehledu změn (česky i anglicky). → *Publish release*.
4. Workflow **Vydání** nejdřív ověří, že tag odpovídá verzi v `Cargo.toml` a že je verze v přehledu změn; pak sestaví balíčky pro všechny platformy a nápovědu a připojí je k vydání. Je-li zapnutá proměnná `NAPOVEDA_PAGES`, zveřejní na GitHub Pages nápovědu a **archiv verzí** (`/vydani/`, `scripts/archiv.py`) – tabulku vydání × platforma s odkazy na balíčky ve vydáních. Binárky se do repozitáře neukládají.

### Průběžné sestavení

Po každém úspěšném CI na `main` přepíše workflow **Průběžné sestavení** (`.github/workflows/prubezne.yml`) předběžné vydání s tagem `prubezne`: balíčky `LogicFlow-prubezne-<platforma>` z právě dokončeného CI (s README, licencí, přehledem změn a `SESTAVENI.txt` s commitem), nápověda v HTML a poznámky ze sekce Nevydáno. Není označené jako nejnovější, takže odkaz *releases/latest* dál vede na oficiální verzi; workflow Vydání tag `prubezne` přeskakuje. Ručně: *Actions → Průběžné sestavení → Run workflow*. Tag `prubezne` se ručně nemaže ani nepřesouvá – dělá to workflow.

## Užitečné příkazy

```sh
cargo run                              # spuštění
cargo test --workspace                 # testy (včetně kontroly nápovědy)
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt
python3 scripts/komentare.py kontrola  # komentáře česky i anglicky
python3 scripts/zmeny.py kontrola      # přehled změn CHANGELOG.md / CHANGELOG.en.md
python3 scripts/formaty.py kontrola    # dokumentace formátů souborů (pip install jsonschema)
python3 scripts/zmeny.py poznamky 0.3.1  # poznámky k vydání z přehledu změn
mdbook serve docs/napoveda/cs --open   # náhled nápovědy (cargo install mdbook --locked)
scripts/snimky-napovedy.sh             # snímky obrazovky pro nápovědu (Linux)
```

Podrobnosti k psaní nápovědy jsou v [docs/napoveda/README.md](docs/napoveda/README.md).
