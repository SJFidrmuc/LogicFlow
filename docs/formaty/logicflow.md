# Formát projektu LogicFlow (`.logicflow`)

**Specifikace formátu 1** · stav: LogicFlow 0.3.1 a nevydané změny (0.4.0) · [English](logicflow.en.md) · [Přehled formátů a historie](README.md) · [JSON Schema](logicflow.schema.json)

## 1. Účel a rozsah

Soubor `.logicflow` obsahuje jeden **projekt**: hlavní obvod (list schématu), definice všech podobvodů, které používá, a nepovinné informace o obvodu (autor, licence…). Stejný tvar má text ve **schránce** (Ctrl+C / Ctrl+V) a text, kterým se do projektu vkládá prvek z knihovny. Objekt **`circuit`** (kapitoly 6–10) je společný i pro knihovnu prvků [`.lflib`](lflib.md).

Kód: `crates/logicflow-core/src/circuit.rs` (`ProjectFile`, `Circuit`, `Component`, `ComponentKind`, `Wire`, `SubcircuitDef`, `CircuitInfo`), `block.rs` (`BlockKind`), `gate.rs` (`GateKind`).

## 2. Závaznost a pojmy

- **MUSÍ / NESMÍ** – povinné; soubor, který pravidlo porušuje, není platný. **MĚL BY / NEMĚL BY** – doporučené; odchylka musí mít důvod. **MŮŽE** – volitelné.
- **Zapisovač** je program, který soubor vytváří (LogicFlow, skript, převodník). **Čtenář** je program, který soubor načítá.
- Sloupec **Od verze** udává první verzi LogicFlow, která pole nebo hodnotu zná. Starší verze s ní neumí pracovat – viz kapitola 13.

## 3. Soubor jako celek

- Text **JSON** (RFC 8259) v kódování **UTF-8** bez BOM. Přípona `.logicflow`.
- Kořenem je objekt (kapitola 4). Pořadí klíčů ani formátování nemají význam; LogicFlow zapisuje klíče v pořadí podle této specifikace s odsazením 2 mezery, aby se soubory dobře porovnávaly v Gitu.
- Čísla jsou celá; **nesmí** mít desetinnou část ani exponent.
- Čtenář **musí ignorovat** klíče, které nezná (kvůli souborům z novějších verzí). Zapisovač **nesmí** přidávat vlastní klíče mimo část `info`.

## 4. Kořenový objekt

| Klíč | Typ | Povinný | Od verze | Význam |
| --- | --- | --- | --- | --- |
| `format` | celé číslo ≥ 1 | ano | 0.1.0 | verze formátu, teď **1** |
| `app` | text | ano | 0.1.0 | program, který soubor uložil, např. `"LogicFlow 0.4.0"` – jen informativní |
| `info` | objekt | ne | 0.4.0 | informace o obvodu (kapitola 5); prázdná se nezapisuje |
| `circuit` | objekt | ano | 0.1.0 | hlavní obvod (kapitola 6) |
| `subcircuits` | pole | ne (výchozí `[]`) | 0.3.0 | definice podobvodů (kapitola 11); prázdné se nezapisuje |

**Postup čtení:** čtenář nejdřív ověří `format`. Je-li vyšší, než zná, **musí** soubor odmítnout s hláškou, že je z novější verze programu (LogicFlow: „Soubor je z novější verze aplikace (formát N), tato verze umí formát 1.“). Soubor s `"type": "logicflow-library"` je knihovna, ne projekt (viz [lflib.md](lflib.md)).

## 5. Informace o obvodu `info` (od 0.4.0)

Všechna pole jsou nepovinná a prázdná se nezapisují. Klíče odpovídají souboru `info.toml` v online knihovně obvodů.

| Klíč | Typ | Od verze | Význam |
| --- | --- | --- | --- |
| `title` | text | 0.4.0 | název obvodu |
| `authors` | pole textů | 0.4.0 | autoři; první je původní autor, další spoluautoři |
| `description` | text | 0.4.0 | co obvod dělá, k čemu slouží |
| `license` | text | 0.4.0 | licence, např. `CC BY 4.0`, `CC BY-SA 4.0`, `CC BY-NC 4.0`, `CC0 1.0`; „neuvedeno“ / „not specified“ nebo prázdné = bez licence |
| `school` | text | 0.4.0 | škola, třída nebo kroužek |
| `created` | datum `RRRR-MM-DD` | 0.4.0 | datum vytvoření |
| `modified` | datum `RRRR-MM-DD` | 0.4.0 | datum poslední úpravy (zapíše se při každém uložení) |
| `app_version` | text | 0.4.0 | verze programu, ve které obvod vznikl, např. `"0.4.0"` |

Další klíče v `info` (z novější verze nebo z online knihovny) **musí** zapisovač, který soubor jen upravuje, zachovat beze změny. LogicFlow to dělá od 0.4.0; verze 0.1–0.3 celou část `info` při uložení zahodí.

## 6. Obvod `circuit`

Jeden list schématu. Stejný objekt je v `circuit` projektu, v každé definici podobvodu a v každém prvku knihovny.

| Klíč | Typ | Povinný | Od verze | Význam |
| --- | --- | --- | --- | --- |
| `components` | pole objektů | ano | 0.1.0 | součástky (kapitola 8) |
| `wires` | pole objektů | ano | 0.1.0 | vodiče (kapitola 10) |
| `next_id` | celé číslo ≥ 0 | ne (výchozí 0) | 0.1.0 | další volné číslo pro novou součástku nebo vodič |

**Identifikátory:** součástky a vodiče jednoho obvodu sdílejí jednu číselnou řadu – `id` **musí** být v rámci obvodu jedinečné mezi součástkami i vodiči (0 až 2³²−1). Čísla nemusí jít po sobě. Čtenář pokračuje v číslování od `max(next_id, nejvyšší id + 1)`; zapisovač **měl by** zapsat `next_id` větší než všechna použitá `id`.

## 7. Souřadnice a otočení

- Body leží na celočíselné **mřížce**, osa y míří **dolů** (jako na obrazovce):

| Klíč | Typ | Povinný | Od verze | Význam |
| --- | --- | --- | --- | --- |
| `x` | celé číslo −2³¹ až 2³¹−1 | ano | 0.1.0 | vodorovná souřadnice, roste doprava |
| `y` | celé číslo −2³¹ až 2³¹−1 | ano | 0.1.0 | svislá souřadnice, roste dolů |

- Každá součástka má **počátek** v bodě `pos`. Vývody leží v pevných místních souřadnicích (kapitola 9.7), které se otočí podle `rotation` a posunou o `pos`:

| Hodnota `rotation` | Od verze | Otočení (po směru hodinových ručiček) | Místní bod (x, y) → světový |
| --- | --- | --- | --- |
| `R0` | 0.1.0 | bez otočení | (pos.x + x, pos.y + y) |
| `R90` | 0.1.0 | o 90° | (pos.x − y, pos.y + x) |
| `R180` | 0.1.0 | o 180° | (pos.x − x, pos.y − y) |
| `R270` | 0.1.0 | o 270° | (pos.x + y, pos.y − x) |

Číslice na zobrazovačích zůstává vždy rovně, stranu vývodů zobrazovače určuje `pins`.

## 8. Součástka

| Klíč | Typ | Povinný | Od verze | Význam |
| --- | --- | --- | --- | --- |
| `id` | celé číslo | ano | 0.1.0 | identifikátor (kapitola 6) |
| `kind` | objekt | ano | 0.1.0 | druh součástky a jeho nastavení (kapitola 9) |
| `pos` | bod | ano | 0.1.0 | poloha počátku značky |
| `rotation` | text | ne (výchozí `R0`) | 0.1.0 | otočení (kapitola 7) |
| `label` | text | ne (výchozí `""`) | 0.1.0 | popisek (A, B, Y1…); u přepínačů, tlačítek a LED v podobvodu tvoří **názvy vývodů** (kapitola 11) |

Stav stisku tlačítka se neukládá. Stav přepínače (`on`) se ukládá.

## 9. Druhy součástek `kind`

Objekt `kind` má vždy klíč **`type`** a podle něj další parametry.

| `type` | Parametry | Vstupy | Výstupy | Od verze | Součástka |
| --- | --- | --- | --- | --- | --- |
| `gate` | `gate`, `inputs` | 1–8 | 1 | 0.1.0 | logické hradlo (9.1) |
| `switch` | `on` (bool) | 0 | 1 | 0.1.0 | přepínač – drží 0 nebo 1 |
| `button` | – | 0 | 1 | 0.1.0 | tlačítko – 1 jen po dobu stisku |
| `constant` | `value` (bool) | 0 | 1 | 0.1.0 | konstanta 0 / 1 |
| `led` | – | 1 | 0 | 0.1.0 | svítivá dioda |
| `flip_flop` | `ff`, `falling`, `initial` | 2–3 | 2 | 0.2.0 | klopný obvod (9.4) |
| `clock` | `period_ms` | 0 | 1 | 0.2.0 | generátor hodin |
| `block` | `block` (objekt) | podle bloku | podle bloku | 0.3.0 | funkční blok (9.5) |
| `seven_segment` | `pins` | 8 | 0 | 0.3.0 | sedmisegmentový zobrazovač (9.3) |
| `hex_display` | `pins` | 4 | 0 | 0.3.0 | hexadecimální zobrazovač (9.3) |
| `subcircuit` | `id`, `inputs`, `outputs` | 0–32 | 0–32 | 0.3.0 | instance podobvodu (9.6) |
| `vcc` | – | 0 | 1 | 0.4.0 | napájení – logická 1 se značkou napájení |
| `gnd` | – | 0 | 1 | 0.4.0 | zem – logická 0 se značkou země |

Čtenář, který `type` nezná, **musí** soubor odmítnout (nesmí součástku tiše vynechat – změnilo by to zapojení).

### 9.1 Hradla (`gate`)

`{"type": "gate", "gate": "Nand", "inputs": 2}` – `inputs` je počet vstupů: u `Buffer` a `Not` **musí** být 1, u ostatních 2–8. Hodnota `gate` se píše s velkým počátečním písmenem:

| `gate` | Od verze | Funkce |
| --- | --- | --- |
| `Buffer` | 0.1.0 | opakovač Y = A |
| `Not` | 0.1.0 | negace Y = Ā |
| `And` | 0.1.0 | logický součin |
| `Or` | 0.1.0 | logický součet |
| `Nand` | 0.1.0 | negovaný součin |
| `Nor` | 0.1.0 | negovaný součet |
| `Xor` | 0.1.0 | lichá parita vstupů (výhradní součet) |
| `Xnor` | 0.1.0 | sudá parita vstupů |

### 9.2 Zdroje signálu

- `switch` – `on`: `true` = výstup 1. **Vstup obvodu** (v podobvodu vývod).
- `button` – při simulaci 1 jen po dobu stisku, jinak 0. **Vstup obvodu**.
- `constant` – `value`: `true` = 1.
- `vcc` / `gnd` (od 0.4.0) – logická 1 / 0 s elektrotechnickou značkou, bez automatického popisku. Spojení zdrojů s různou pevnou úrovní (VCC, GND, konstanty) je **zkrat napájení**.
- `clock` – `period_ms`: perioda v milisekundách, celé číslo ≥ 1; LogicFlow nabízí 10–10 000 ms (0,1–100 Hz) a hodnoty mimo rozsah při úpravě ořízne. Na začátku simulace dávají hodiny 0.

### 9.3 Zobrazovače

- `led` – **výstup obvodu** (v podobvodu vývod).
- `seven_segment` – vstupy **a, b, c, d, e, f, g, dp** v tomto pořadí.
- `hex_display` – vstupy **D0–D3** (D0 nejnižší bit); zobrazí 0–F.

Klíč `pins` (nepovinný, výchozí `left`) určuje stranu s vývody:

| `pins` | Od verze | Vývody |
| --- | --- | --- |
| `left` | 0.3.0 | vlevo, shora dolů |
| `right` | 0.3.0 | vpravo, shora dolů |
| `top` | 0.3.0 | nahoře, zleva doprava |
| `bottom` | 0.3.0 | dole, zleva doprava |

### 9.4 Klopné obvody (`flip_flop`)

`{"type": "flip_flop", "ff": "jk", "falling": false, "initial": false}`

| `ff` | Od verze | Vstupy (shora) | Řízení |
| --- | --- | --- | --- |
| `rs` | 0.2.0 | S, R | asynchronní |
| `d_latch` | 0.2.0 | D, C | úrovní – „průhledný“, dokud C = 1 |
| `d` | 0.2.0 | D, C | hranou C |
| `jk` | 0.2.0 | J, C, K | hranou C |
| `t` | 0.2.0 | T, C | hranou C |

Výstupy jsou vždy **Q** (nahoře) a **Q̄** (dole). `falling` (nepovinný, výchozí `false`): `true` = spouští sestupná hrana, jinak náběžná; u `rs` a `d_latch` se nepoužije. `initial` (nepovinný, výchozí `false`): stav Q po spuštění simulace.

### 9.5 Funkční bloky (`block`)

`{"type": "block", "block": {"kind": "counter", "bits": 4, "down": false}}` – vnořený objekt `block` má klíč **`kind`** a parametr. Parametr **musí** být v uvedeném rozsahu. Bity se počítají od nejnižšího (index 0 nahoře).

| `kind` | Parametry (rozsah) | Vstupy shora | Výstupy shora | Od verze |
| --- | --- | --- | --- | --- |
| `mux` | `select` 1–3 | S0…S(s−1), D0…D(2ˢ−1), EN | Y | 0.3.0 |
| `decoder` | `bits` 1–3 | A0…A(n−1), EN | Y0…Y(2ⁿ−1) | 0.3.0 |
| `encoder` | `bits` 2–3 | I0…I(2ⁿ−1) | A0…A(n−1), V | 0.3.0 |
| `adder` | `bits` 1–4 | A0…A(n−1), B0…B(n−1), CI | S0…S(n−1), CO | 0.3.0 |
| `comparator` | `bits` 1–4 | A0…A(n−1), B0…B(n−1) | A<B, A=B, A>B | 0.3.0 |
| `bcd_to7_seg` | – | D0…D3 | a…g | 0.3.0 |
| `counter` | `bits` 2–8, `down` (bool) | R, EN, C | Q0…Q(n−1), CO | 0.3.0 |
| `register` | `bits` 2–8 | D0…D(n−1), R, C | Q0…Q(n−1) | 0.3.0 |
| `shift_register` | `bits` 2–8 | SI, R, C | Q0…Q(n−1) | 0.3.0 |

U sekvenčních bloků (`counter`, `register`, `shift_register`) je hodinový vstup C poslední. Nepřipojený vstup R se chová jako 0, nepřipojený EN jako 1.

### 9.6 Instance podobvodu (`subcircuit`)

`{"type": "subcircuit", "id": 1, "inputs": 3, "outputs": 2}` – `id` odkazuje na definici v `subcircuits` (v knihovně na prvek se stejným `id`, viz [lflib.md](lflib.md)). `inputs` a `outputs` jsou kopie rozhraní definice (0–32), aby šlo značku nakreslit bez definice. Zapisovač **musí** zapsat počty podle definice; čtenář je **měl by** podle definice přepočítat (LogicFlow to dělá při uložení). Pořadí vývodů viz kapitola 11.

### 9.7 Poloha vývodů

Místní souřadnice pro `R0` (otočení viz kapitola 7). Vývody se počítají shora dolů (u `top`/`bottom` zleva doprava) v pořadí z tabulek výše; *k* = 0, 1, 2…

| Součástka | Vstupy | Výstupy |
| --- | --- | --- |
| `gate` s *n* vstupy | (−3, −(*n*−1) + 2*k*) | (3, 0) |
| `switch`, `button`, `constant` | – | (2, 0) |
| `vcc` | – | (0, 1) |
| `gnd` | – | (0, −1) |
| `led` | (−2, 0) | – |
| `flip_flop` `jk` | (−3, −2), (−3, 0), (−3, 2) | Q (3, −2), Q̄ (3, 2) |
| `flip_flop` ostatní | (−3, −1), (−3, 1) | Q (3, −1), Q̄ (3, 1) |
| `clock` | – | (3, 0) |
| `block`, `subcircuit` (obdélník), *m* = max(vstupy, výstupy, 1) | (−4, 2 − *m* + 2*k*) | (4, 2 − *m* + 2*k*) |
| `seven_segment` `left` / `right` | (∓4, −7 + 2*k*) | – |
| `seven_segment` `top` / `bottom` | (−7 + 2*k*, ∓7) | – |
| `hex_display` `left` / `right` | (∓4, −3 + 2*k*) | – |
| `hex_display` `top` / `bottom` | (−3 + 2*k*, ∓5) | – |

(U `∓` platí horní znaménko pro `left` / `top`.)

## 10. Vodiče a propojení

| Klíč | Typ | Povinný | Od verze | Význam |
| --- | --- | --- | --- | --- |
| `id` | celé číslo | ano | 0.1.0 | identifikátor (kapitola 6) |
| `a` | bod | ano | 0.1.0 | první konec |
| `b` | bod | ano | 0.1.0 | druhý konec |

- Vodič **musí** být vodorovný (`a.y = b.y`) nebo svislý (`a.x = b.x`); lomený vodič se zapisuje jako několik úseků. Zapisovač **neměl by** zapisovat vodiče nulové délky.
- Vodiče se propojují **v koncových bodech**. Konec vodiče, který leží uvnitř jiného vodiče, se s ním také propojí (**odbočka „T“**). Vodiče, které se jen **kříží**, propojené **nejsou**.
- **Vývod** součástky se připojí k vodiči, jehož konec v něm leží (nebo který jím prochází v místě, kde končí jiný vodič). Vodič, který přes vývod jen prochází, se s ním nespojí.
- Vývody různých součástek ve stejném bodě jsou propojené i bez vodiče.
- Skupina propojených vodičů a vývodů je jedna **síť**. Nepřipojený vstup má hodnotu Z (vysoká impedance), kterou hradla čtou jako X.

## 11. Podobvody `subcircuits` (od 0.3.0)

| Klíč | Typ | Povinný | Od verze | Význam |
| --- | --- | --- | --- | --- |
| `id` | celé číslo | ano | 0.3.0 | číslo definice – na něj odkazují instance; jedinečné v souboru |
| `name` | text | ano | 0.3.0 | název na značce a v Hierarchii |
| `circuit` | objekt | ano | 0.3.0 | zapojení podobvodu (kapitola 6) |

- **Rozhraní** podobvodu tvoří součástky uvnitř: **vstupy** = `switch` a `button`, **výstupy** = `led`. Každá skupina se seřadí **přirozeně** podle `label`: popisek se rozdělí před první číslicí na text a číslo, porovná se text (podle Unicode), pak číslo, nakonec celý popisek – takže `A2` < `A10` < `B`. První v pořadí je horní vývod značky. Platí nejvýš 32 vstupů a 32 výstupů; další se nepřipojí.
- Popisky vývodů **měly by** být jedinečné a neprázdné.
- Instance odkazuje jen na definice ve stejném souboru. Podobvod **nesmí** obsahovat (ani nepřímo) sám sebe. Simuluje se vnoření do hloubky **16**; hlubší instance zůstanou nezapojené.

## 12. Pravidla pro zapisovatele

LogicFlow při uložení:

- zapíše `format` = aktuální verzi formátu a `app` = `"LogicFlow X.Y.Z"`;
- u instancí podobvodů přepočítá `inputs` / `outputs` podle definic;
- v části `info` zapíše `modified` (a u nového obvodu `created` a `app_version`), prázdné hodnoty vynechá a neznámé klíče zachová;
- vynechá prázdné `info` a prázdné `subcircuits`.

Jiný zapisovač **musí** dodržet povinná pole a rozsahy z kapitol 4–11; ostatní body **měl by**.

## 13. Kompatibilita s verzemi programu

| Soubor obsahuje | LogicFlow 0.1 | 0.2 | 0.3 | 0.4+ |
| --- | --- | --- | --- | --- |
| hradla, přepínače, tlačítka, konstanty, LED | ano | ano | ano | ano |
| klopné obvody, hodiny | odmítne | ano | ano | ano |
| funkční bloky, zobrazovače, podobvody | odmítne | odmítne | ano | ano |
| `vcc`, `gnd` | odmítne | odmítne | odmítne | ano |
| část `info` | otevře, při uložení zahodí | totéž | totéž | ano |
| `format` vyšší než 1 | odmítne | odmítne | odmítne | odmítne |

- **Nejnižší verze**, která soubor otevře = nejvyšší „Od verze“ ze všech `type`, `gate`, `ff`, `kind` bloku a `pins` použitých v souboru.
- Nová součástka nebo nová hodnota v budoucích verzích nezvyšuje `format`, ale starší verze takový soubor odmítne (viz kapitola 9). Nová nepovinná pole starší verze přeskočí.
- Uvnitř formátu 1 se klíče ani hodnoty nepřejmenovávají ani neodebírají a každé nové pole má výchozí hodnotu – soubor z libovolné starší verze proto novější verze vždy otevře.

## 14. Příklad

Invertor z hradla NAND jako podobvod: přepínač A → podobvod „Invertor z NAND“ → LED Y. Uvnitř podobvodu jsou oba vstupy hradla NAND spojené odbočkou „T“ a připojené k přepínači A. Test `formaty.rs` ověřuje, že se příklad načte a že Y = Ā. (Příklad je v obou jazykových verzích specifikace stejný.)

<!-- příklad: logicflow -->
```json
{
  "format": 1,
  "app": "LogicFlow 0.4.0",
  "info": {
    "title": "Invertor z NAND",
    "authors": ["Mgr. Stanislav Jan Fidrmuc"],
    "license": "CC BY 4.0",
    "created": "2026-11-27",
    "modified": "2026-11-27",
    "app_version": "0.4.0"
  },
  "circuit": {
    "components": [
      { "id": 0, "kind": { "type": "switch", "on": false }, "pos": { "x": 0, "y": 0 }, "rotation": "R0", "label": "A" },
      { "id": 1, "kind": { "type": "subcircuit", "id": 1, "inputs": 1, "outputs": 1 }, "pos": { "x": 8, "y": -1 }, "rotation": "R0", "label": "P1" },
      { "id": 2, "kind": { "type": "led" }, "pos": { "x": 16, "y": 0 }, "rotation": "R0", "label": "Y" }
    ],
    "wires": [
      { "id": 3, "a": { "x": 2, "y": 0 }, "b": { "x": 4, "y": 0 } },
      { "id": 4, "a": { "x": 12, "y": 0 }, "b": { "x": 14, "y": 0 } }
    ],
    "next_id": 5
  },
  "subcircuits": [
    {
      "id": 1,
      "name": "Invertor z NAND",
      "circuit": {
        "components": [
          { "id": 0, "kind": { "type": "switch", "on": false }, "pos": { "x": 0, "y": 0 }, "rotation": "R0", "label": "A" },
          { "id": 1, "kind": { "type": "gate", "gate": "Nand", "inputs": 2 }, "pos": { "x": 8, "y": 0 }, "rotation": "R0", "label": "G1" },
          { "id": 2, "kind": { "type": "led" }, "pos": { "x": 14, "y": 0 }, "rotation": "R0", "label": "Y" }
        ],
        "wires": [
          { "id": 3, "a": { "x": 2, "y": 0 }, "b": { "x": 4, "y": 0 } },
          { "id": 4, "a": { "x": 4, "y": -1 }, "b": { "x": 4, "y": 1 } },
          { "id": 5, "a": { "x": 4, "y": -1 }, "b": { "x": 5, "y": -1 } },
          { "id": 6, "a": { "x": 4, "y": 1 }, "b": { "x": 5, "y": 1 } },
          { "id": 7, "a": { "x": 11, "y": 0 }, "b": { "x": 12, "y": 0 } }
        ],
        "next_id": 8
      }
    }
  ]
}
```
