# LogicFlow

Simulátor logických obvodů pro výuku číslicové techniky. Moderní nástupce programu DigitalWorks: česky, s rozhraním ve stylu Blenderu, pro Windows, macOS, Linux i Raspberry Pi.

![Pracovní plocha Simulace: schéma, vlastnosti a časový diagram](docs/screenshots/simulace.png)

> **Stav:** milník M0 (základ projektu). Aplikace zatím zobrazuje ukázkový obvod se dvěma přepínači, hradly AND a OR a dvěma LED. Kreslení vlastních schémat přijde ve verzi 0.1.

## Co už funguje

- Okno rozdělené na **oblasti** jako v Blenderu. Každá oblast má v záhlaví volbu typu editoru, dá se přetáhnout za záhlaví a měnit její velikost.
- **Pracovní plochy** Návrh, Simulace, Analýza a Paměť, přepínání i přes Ctrl+PageUp / Ctrl+PageDown.
- **Editor schématu** s mřížkou, plynulým zoomem kolečkem myši a posunem prostředním nebo pravým tlačítkem. Klávesa Home zobrazí celé schéma.
- **Ukázkový obvod**, který vyhodnocuje simulační jádro. Kliknutím na přepínač A nebo B změníte vstup.
- **Časový diagram**, který zaznamenává změny signálů. Mezerník simulaci pozastaví nebo spustí.
- **Pravdivostní tabulka** se zvýrazněným aktuálním řádkem.
- Čeština a angličtina (přepínač CS / EN vpravo nahoře).

## Plán

| Verze | Obsah | Termín (orientačně) |
| --- | --- | --- |
| 0.1 | kreslení schémat, hradla, přepínače, LED, ukládání | 02–03/2027 |
| 0.2 | klopné obvody, hodiny, časový diagram, krokování – pilot ve třídě | 04/2027 |
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

Testy: `cargo test --workspace`.

### Raspberry Pi

Podporované jsou Raspberry Pi 4 a 5 s 64bitovým Raspberry Pi OS (Debian 13 „Trixie“). Hotový program pro Raspberry Pi sestavuje automaticky GitHub Actions; najdete ho u každého běhu CI jako artefakt `logicflow-linux-arm64-raspberry-pi`. Sestavení přímo na Raspberry Pi funguje stejně jako výše, jen trvá déle.

## Struktura projektu

| Složka | Obsah |
| --- | --- |
| `crates/logicflow-core` | simulační jádro bez grafického rozhraní (logické hodnoty 0/1/X/Z, hradla) |
| `crates/logicflow` | aplikace: okno, oblasti, editory, překlady |
| `.github/workflows` | automatické sestavení a testy pro 6 platforem |

## Licence

[MIT](LICENSE) © 2026 Stanislav Jan Fidrmuc
