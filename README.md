# LogicFlow

Simulátor logických obvodů pro výuku číslicové techniky. Moderní nástupce programu DigitalWorks: česky, s rozhraním ve stylu Blenderu, pro Windows, macOS, Linux i Raspberry Pi.

![Čítač 0–7 z klopných obvodů T v pracovní ploše Simulace](docs/screenshots/simulace.png)

> **Stav:** verze 0.2 (milník M2). Kreslení obvodů z hradel, klopných obvodů a generátorů hodin, simulace v reálném čase s krokováním, časový diagram, pravdivostní tabulka, ukládání souborů a vestavěná nápověda.

## Stažení

Hotové programy jsou na stránce **[Releases → nejnovější verze](https://github.com/SJFidrmuc/LogicFlow/releases/latest)** (bez přihlášení, bez instalace).

| Systém | Soubor |
| --- | --- |
| Windows (běžné PC) | `LogicFlow-…-windows-x64.zip` |
| Windows na procesoru ARM | `LogicFlow-…-windows-arm64.zip` |
| macOS (Apple M1 a novější) | `LogicFlow-…-macos-arm64.tar.gz` |
| macOS (Intel) | `LogicFlow-…-macos-x64.tar.gz` |
| Linux (PC) | `LogicFlow-…-linux-x64.tar.gz` |
| Raspberry Pi 4 a 5 (64bitový Raspberry Pi OS) | `LogicFlow-…-raspberry-pi-arm64.tar.gz` |

- **Windows:** rozbalte ZIP a spusťte `logicflow.exe`. Pokud Windows zobrazí varování, viz [níže](#windows-hlásí-nebezpečnou-aplikaci).
- **Linux a Raspberry Pi:** `tar -xzf LogicFlow-…tar.gz`, pak `./logicflow` ve vzniklé složce.
- **macOS:** po rozbalení při prvním spuštění klikněte na `logicflow` pravým tlačítkem → Otevřít (program zatím není podepsaný).

## Co umí

- **Kreslení schémat** – součástky se přidávají přes Shift+A nebo z knihovny, vodiče se kreslí tažením z vývodu. Vodiče se při přesunu součástky samy přepojí.
- **Součástky** – přepínač, tlačítko, konstanty 0 a 1, generátor hodin (0,2–10 Hz), hradla BUF, NOT, AND, OR, NAND, NOR, XOR, XNOR (2–8 vstupů), LED. Značky podle ČSN EN 60617.
- **Klopné obvody** RS, D řízený úrovní, D, JK a T řízené náběžnou nebo sestupnou hranou, s výstupy Q a Q̄ a volitelným počátečním stavem.
- **Událostní simulace** se zpožděním hradel a hodnotami 0, 1, X (neurčitá, kolize) a Z (nepřipojeno). Simulátor pozná kmitající obvod a obvody s pamětí (klopný obvod z hradel) si pamatují stav i po úpravě schématu.
- **Krokování** – při pozastavené simulaci (mezerník) se šipkou vpravo provádí jeden takt; je vidět, jak signál prochází hradly.
- **Časový diagram** vstupů a výstupů a **pravdivostní tabulka** spočítaná simulací všech kombinací vstupů.
- **Soubory** `.logicflow` (čitelný JSON), vlastní prohlížeč souborů, Zpět/Znovu, vestavěné příklady (hradla, poloviční sčítačka, klopný obvod RS z hradel, klopný obvod D s hodinami, čítač 0–7, posuvný registr).
- Okno rozdělené na **oblasti** a **pracovní plochy** (Návrh, Simulace, Analýza, Paměť) jako v Blenderu. Oblasti jde rozdělovat a zavírat (nabídka **Oblast** v záhlaví nebo pravé tlačítko na záhlaví) a v každé zvolit typ editoru.
- **Nápověda** v aplikaci (F1 nebo nabídka Nápověda) – postup pro první obvod, popis ovládání, součástek, simulace a klávesových zkratek. Nápovědu jde zobrazit i jako oblast vedle schématu.
- Čeština a angličtina.

| Návrh | Analýza |
| --- | --- |
| ![Nabídka Shift+A](docs/screenshots/navrh.png) | ![Pravdivostní tabulka poloviční sčítačky](docs/screenshots/analyza.png) |

## Ovládání

| Klávesa / myš | Akce |
| --- | --- |
| F1 | nápověda |
| Shift+A | nabídka Přidat součástku |
| tažení z vývodu (Ctrl+tažení odkudkoli) | nakreslit vodič |
| klik na přepínač / držení tlačítka | změna vstupu |
| klik, Shift+klik, tažení v prázdném místě | výběr, přidání do výběru, výběr obdélníkem |
| tažení součástky, G | přesun výběru |
| R | otočit o 90° (i při umisťování) |
| X, Delete | smazat výběr |
| Shift+D | duplikovat výběr |
| A / Alt+A | vybrat vše / zrušit výběr |
| kolečko, prostřední tlačítko, Home | zoom, posun, celé schéma |
| mezerník, šipka vpravo | spustit/pozastavit simulaci, jeden takt |
| Ctrl+R | restartovat simulaci (klopné obvody do počátečního stavu) |
| Ctrl+Z, Ctrl+Shift+Z | zpět, znovu |
| Ctrl+N, Ctrl+O, Ctrl+S, Ctrl+Shift+S | nový, otevřít, uložit, uložit jako |
| Ctrl+PageUp / PageDown | předchozí / další pracovní plocha |

## Windows hlásí „nebezpečnou“ aplikaci

Program zatím není digitálně podepsaný, proto ho Windows SmartScreen po stažení označí jako nerozpoznanou aplikaci. Zdrojový kód je veřejný a soubory sestavuje automaticky GitHub.

1. V okně *Systém Windows ochránil váš počítač* klikněte na **Další informace** a pak na **Přesto spustit**.
2. Aby se upozornění neopakovalo: před rozbalením klikněte na stažený ZIP pravým tlačítkem → **Vlastnosti** → zaškrtněte **Odblokovat** → OK.

Ve škole může program povolit správce sítě pro všechny počítače najednou. Podepsaná verze (SignPath Foundation nebo Microsoft Store) je v plánu.

## Plán

| Verze | Obsah | Termín (orientačně) |
| --- | --- | --- |
| ~~0.1~~ | kreslení schémat, hradla, přepínače, LED, ukládání | hotovo |
| ~~0.2~~ | klopné obvody, generátor hodin, nápověda – připraveno na pilot ve třídě | hotovo |
| 0.3 | podobvody, čítače, registry, MUX, dekodéry, sedmisegmentovka | 05/2027 |
| 0.4 | paměti ROM, RAM, EEPROM, hex editor | 06/2027 |
| 0.5 | export do VHDL | 07–08/2027 |
| 0.6 | obvody 74xx, příručka, vzorové úlohy | 08/2027 |
| 1.0 | verze pro výuku | 09/2027 |

## Sestavení ze zdrojového kódu

Potřebujete [Rust](https://rustup.rs) 1.95 nebo novější.

```sh
git clone https://github.com/SJFidrmuc/LogicFlow
cd LogicFlow
cargo run --release
```

Soubor obvodu jde otevřít i z příkazové řádky: `logicflow muj-obvod.logicflow`. Testy: `cargo test --workspace`.

### Raspberry Pi

Podporované jsou Raspberry Pi 4 a 5 s 64bitovým Raspberry Pi OS (Debian 13 „Trixie“). Hotový program je ke stažení na stránce Releases (viz Stažení); sestavuje se v Debianu 13, aby odpovídal knihovnám Raspberry Pi OS.

### Vydání nové verze

V GitHubu zvolte *Releases → Draft a new release*, zadejte nový tag (např. `v0.3.0`) a klikněte na *Publish release*. Workflow *Vydání* pak během několika minut sestaví program pro všech šest platforem a soubory sám připojí k vydání.

## Struktura projektu

| Složka | Obsah |
| --- | --- |
| `crates/logicflow-core` | jádro bez grafického rozhraní: model obvodu, netlist, událostní simulátor, formát souboru |
| `crates/logicflow` | aplikace: okno, oblasti, editor schématu, panely, překlady |
| `.github/workflows` | automatické sestavení a testy pro 6 platforem |

## Licence

[MIT](LICENSE) © 2026 Stanislav Jan Fidrmuc
