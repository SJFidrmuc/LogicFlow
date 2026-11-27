# Formát knihovny prvků LogicFlow (`.lflib`)

**Specifikace formátu 1** · stav: LogicFlow 0.3.1 a nevydané změny (0.4.0) · [English](lflib.en.md) · [Přehled formátů a historie](README.md) · [JSON Schema](lflib.schema.json)

## 1. Účel a rozsah

Knihovna je soubor hotových podobvodů – **prvků** – které jde vložit do libovolného projektu. Program je ukazuje v Knihovně součástek pod nadpisem Knihovny prvků: vestavěnou knihovnu, osobní knihovny (`Dokumenty/LogicFlow/Knihovny`) a knihovny ve sdílených složkách. Uživatelský přehled a vzorové soubory jsou v [docs/knihovny](../knihovny/README.md).

Formát `.lflib` je **od verze 0.4.0**. Zapojení prvků používá objekt `circuit` z formátu projektu – kapitoly 6–11 specifikace [logicflow.md](logicflow.md) platí beze změny a tento dokument je neopakuje.

Kód: `crates/logicflow-core/src/library.rs` (`Library`, `LibraryInfo`, `LibraryItem`, `LibraryError`, `CATEGORIES`).

## 2. Závaznost a pojmy

Jako v [logicflow.md](logicflow.md), kapitola 2 (MUSÍ / MĚL BY / MŮŽE, zapisovač, čtenář, Od verze). Navíc:

- **Prvek** – jeden podobvod v knihovně s názvem, popisem, autorem, licencí a verzí.
- **Identita** (`uid`) – stálé označení knihovny nebo prvku, které se nemění při přejmenování ani úpravě.
- **Číslo** (`id`) – číslo prvku v souboru, na které odkazují instance uvnitř jiných prvků téže knihovny.

## 3. Soubor jako celek

- Text **JSON** v kódování **UTF-8** bez BOM, přípona `.lflib`. Ostatní pravidla jako u projektu ([logicflow.md](logicflow.md), kapitola 3): čísla celá, na pořadí klíčů a formátování nezáleží, LogicFlow odsazuje 2 mezerami.
- Knihovna se od projektu pozná podle `"type": "logicflow-library"`.
- Čtenář **musí** ignorovat neznámé klíče. LogicFlow 0.4 je při uložení knihovny nezachová.

## 4. Kořenový objekt

| Klíč | Typ | Povinný | Od verze | Význam |
| --- | --- | --- | --- | --- |
| `format` | celé číslo ≥ 1 | ano | 0.4.0 | verze formátu knihovny, teď **1** (nezávislá na formátu projektu) |
| `type` | text | ano | 0.4.0 | vždy `logicflow-library` |
| `app` | text | ne | 0.4.0 | program, který soubor uložil, např. `"LogicFlow 0.4.0"` – jen informativní |
| `library` | objekt | ano | 0.4.0 | hlavička knihovny (kapitola 5) |
| `items` | pole objektů | ne (výchozí `[]`) | 0.4.0 | prvky (kapitola 6) v pořadí, v jakém je ukazuje panel v rámci kategorie |

**Postup čtení:** (1) text musí být JSON; (2) `type` musí být `logicflow-library`, jinak soubor není knihovna; (3) je-li `format` vyšší, než čtenář zná, **musí** soubor odmítnout s vysvětlením – teprve potom se čte zbytek struktury, protože novější formát může mít jinou stavbu; (4) kontroly z kapitoly 9.

## 5. Hlavička `library`

| Klíč | Typ | Povinný | Od verze | Význam |
| --- | --- | --- | --- | --- |
| `uid` | UUID | ano | 0.4.0 | identita knihovny (kapitola 7.1) |
| `name` | text | ano | 0.4.0 | název knihovny v jazyce autora |
| `description` | text | ne | 0.4.0 | popis knihovny |
| `translations` | objekt | ne | 0.4.0 | překlady názvu a popisu (kapitola 7.3); prázdný se nezapisuje |
| `author` | text | ne | 0.4.0 | autor nebo správce knihovny |
| `license` | text | ne | 0.4.0 | licence knihovny (kapitola 7.4) |
| `version` | celé číslo ≥ 1 | ne (výchozí 1) | 0.4.0 | verze knihovny – zvyšuje se při každém uložení |
| `created` | datum | ne | 0.4.0 | datum vytvoření `RRRR-MM-DD` |
| `modified` | datum | ne | 0.4.0 | datum posledního uložení `RRRR-MM-DD` |

## 6. Prvek (`items[]`)

| Klíč | Typ | Povinný | Od verze | Význam |
| --- | --- | --- | --- | --- |
| `uid` | UUID | ano | 0.4.0 | identita prvku (kapitola 7.1) – podle ní se pozná, ze kterého prvku vznikla kopie v projektu |
| `id` | celé číslo ≥ 0 | ano | 0.4.0 | číslo prvku v souboru; na něj odkazují instance `{"type": "subcircuit", "id": …}` v jiných prvcích |
| `name` | text | ano | 0.4.0 | název na značce, v jazyce autora |
| `category` | text | ne | 0.4.0 | klíč vestavěné kategorie (kapitola 8) nebo vlastní text |
| `description` | text | ne | 0.4.0 | popis – co prvek dělá, vzorce, odkaz na úlohu |
| `translations` | objekt | ne | 0.4.0 | překlady názvu a popisu (kapitola 7.3) |
| `author` | text | ne | 0.4.0 | autor prvku |
| `license` | text | ne | 0.4.0 | licence prvku (kapitola 7.4) |
| `version` | celé číslo ≥ 1 | ne (výchozí 1) | 0.4.0 | verze prvku – zvyšuje se při každé úpravě zapojení nebo popisu |
| `created` | datum | ne | 0.4.0 | datum vytvoření |
| `modified` | datum | ne | 0.4.0 | datum poslední úpravy |
| `inputs` | pole textů | ne | 0.4.0 | názvy vstupů – **jen informativní** (kapitola 6.1) |
| `outputs` | pole textů | ne | 0.4.0 | názvy výstupů – **jen informativní** |
| `circuit` | objekt | ano | 0.4.0 | zapojení prvku – objekt `circuit` z [logicflow.md](logicflow.md), kapitola 6 |

### 6.1 Rozhraní prvku

Rozhraní (vývody značky) se určuje stejně jako u podobvodu projektu ([logicflow.md](logicflow.md), kapitola 11): vstupy = přepínače a tlačítka, výstupy = LED uvnitř `circuit`, seřazené přirozeně podle popisku, nejvýš 32 na každé straně. Pole `inputs` a `outputs` jsou kopie těchto názvů pro čtenáře souboru a web knihovny: zapisovač je **měl by** zapsat, čtenář je **nesmí** používat místo zapojení (LogicFlow je při načtení nepoužije a při uložení přepočítá).

### 6.2 Prvky, které používají jiné prvky

Instance `subcircuit` uvnitř prvku odkazuje svým `id` na **číslo jiného prvku téže knihovny**. Knihovna nemá část `subcircuits` – její roli hrají prvky samy. Odkaz na číslo, které v knihovně není, je chyba; prvek **nesmí** obsahovat (ani nepřímo) sám sebe. Při vložení do projektu se vloží i všechny prvky, které vkládaný prvek používá.

## 7. Společné datové typy

### 7.1 Identita `uid`

- Text ve tvaru UUID `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` (8-4-4-4-12 šestnáctkových číslic). Zapisovač **měl by** používat náhodné UUID verze 4 zapsané malými písmeny; čtenář porovnává **bez ohledu na velikost písmen**.
- `uid` prvků **musí** být v knihovně jedinečné. `uid` se **nesmí** změnit při přejmenování, úpravě zapojení ani přesunu do jiné kategorie. Nový prvek – i vzniklý jako varianta kopií jiného – dostane nové `uid`.
- `uid` knihovny označuje knihovnu jako celek a mezi verzemi souboru se nemění.

### 7.2 Číslo `id`

Celé číslo, jedinečné mezi prvky knihovny. Čísla nemusí jít po sobě. Nový prvek dostane číslo o 1 vyšší, než je nejvyšší použité. Číslo prvku se nemá měnit, protože na něj odkazují jiné prvky.

### 7.3 Překlady `translations`

Objekt, jehož klíče jsou kódy jazyka **ISO 639-1** (`en`, `cs`, `sk`, `uk`, `de` …), hodnoty objekty s nepovinnými klíči `name` a `description`:

```json
"translations": { "en": { "name": "1-bit full adder", "description": "Full adder with carry in and carry out." } }
```

| Klíč | Typ | Od verze | Význam |
| --- | --- | --- | --- |
| `name` | text | 0.4.0 | přeložený název; prázdný = použije se hlavní |
| `description` | text | 0.4.0 | přeložený popis; prázdný = použije se hlavní |

Program ukáže text v jazyce rozhraní, pokud překlad existuje a není prázdný, jinak hlavní `name` / `description`. Prázdné hodnoty se nezapisují.

### 7.4 Licence a data

- `license` je volný text. Program nabízí `CC BY 4.0`, `CC BY-SA 4.0`, `CC BY-NC 4.0`, `CC0 1.0` a „neuvedeno“ / „not specified“ (= bez licence, stejně jako prázdný text). **Výchozí licence nových knihoven a prvků je `CC BY 4.0`** (stejně jako online knihovna obvodů).
- Data jsou ve tvaru `RRRR-MM-DD` nebo prázdná.

## 8. Kategorie

`category` je klíč vestavěné kategorie, nebo vlastní text (uloží se, jak ho autor napsal, bez mezer na okrajích). Panel řadí vestavěné kategorie v pořadí tabulky a vlastní za ně.

| Klíč | Česky | English | Od verze |
| --- | --- | --- | --- |
| `combinational` | Kombinační obvody | Combinational circuits | 0.4.0 |
| `arithmetic` | Aritmetika | Arithmetic | 0.4.0 |
| `code-converters` | Převodníky kódů | Code converters | 0.4.0 |
| `sequential` | Sekvenční obvody | Sequential circuits | 0.4.0 |
| `memory` | Paměti | Memories | 0.4.0 |
| `applications` | Aplikace | Applications | 0.4.0 |
| `other` | Ostatní | Other | 0.4.0 |

Čtenář rozpozná vestavěnou kategorii podle klíče i podle českého či anglického názvu bez ohledu na velikost písmen a mezery okolo. Zapisovač **měl by** vestavěnou kategorii zapsat jako klíč (LogicFlow převede `Aritmetika` → `arithmetic`). Neznámý klíč z novější verze se zobrazí jako vlastní kategorie.

## 9. Kontroly při načtení

Čtenář **musí** knihovnu odmítnout, když:

| Chyba (`LibraryError`) | Hláška LogicFlow (cs / en) |
| --- | --- |
| text není JSON nebo neodpovídá struktuře | Soubor knihovny nejde přečíst: … / The library file cannot be read: … |
| chybí `"type": "logicflow-library"` | Soubor není knihovna LogicFlow. / The file is not a LogicFlow library. |
| `format` je vyšší než známý | Knihovna je z novější verze aplikace (formát N), tato verze umí formát 1. / The library is from a newer version of the application (format N), this version supports format 1. |
| `uid` knihovny nebo prvku není UUID | Neplatná identita (uid) „…“. / Invalid identity (uid) “…”. |
| dva prvky mají stejné `uid` | Dva prvky mají stejnou identitu „…“. / Two items have the same identity “…”. |
| dva prvky mají stejné `id` | Dva prvky mají stejné číslo N. / Two items have the same number N. |
| instance odkazuje na číslo, které v knihovně není | Prvek „…“ používá prvek číslo N, který v knihovně není. / Item “…” uses item number N, which is not in the library. |
| prvek obsahuje sám sebe | Prvek „…“ obsahuje sám sebe. / Item “…” contains itself. |

Vadná knihovna se v panelu ukáže jako chyba; program nespadne a ostatní knihovny načte.

## 10. Pravidla pro zapisovatele

LogicFlow při uložení knihovny:

- zapíše `format` = 1, `type` = `logicflow-library`, `app` = `"LogicFlow X.Y.Z"`;
- převede vestavěné kategorie na klíče, přepočítá `inputs` / `outputs` ze zapojení a počty vývodů instancí podle prvků, na které odkazují;
- zvýší `version` knihovny a zapíše `modified`; u upraveného prvku zvýší jeho `version` a zapíše `modified`;
- nové `uid` vytvoří z 16 náhodných bajtů (UUID verze 4); datum a náhodu dodává aplikace, jádro je nepotřebuje (kvůli webové verzi).

## 11. Vložení prvku do projektu (informativní)

Prvek se do projektu vkládá jako **kopie**, stejně jako Ctrl+V: jádro připraví text ve tvaru [`.logicflow`](logicflow.md) s jednou instancí prvku a definicemi všech prvků, které používá (`Library::insertion_text`, názvy v jazyce rozhraní), a projekt ho vloží jako ze schránky. Záznam původu (knihovna, `uid` a `version` prvku) u vloženého podobvodu se připravuje (etapa K1, krok 4) – přibude jako nepovinné pole ve formátu projektu s Od verze podle vydání.

## 12. Kompatibilita s verzemi programu

- Knihovny umí LogicFlow **od 0.4.0**; starší verze soubor `.lflib` neotevřou (nemají knihovny).
- Zapojení prvků podléhá stejným pravidlům jako projekt ([logicflow.md](logicflow.md), kapitola 13): nejnižší verze, která knihovnu načte, je nejvyšší „Od verze“ ze všech součástek ve všech prvcích – ale nejméně 0.4.0. Knihovna s neznámou součástkou se v panelu ukáže jako chyba.
- Uvnitř formátu 1 se klíče ani hodnoty nepřejmenovávají ani neodebírají a každé nové pole má výchozí hodnotu.

## 13. Příklad

Dva prvky: invertor z hradla NAND (číslo 1) a opakovač ze dvou invertorů (číslo 2), který prvek 1 používá dvakrát. Test `formaty.rs` ověřuje, že se příklad načte a oba prvky počítají správně. Úplnější vzor je [docs/knihovny/vzor.lflib](../knihovny/vzor.lflib). (Příklad je v obou jazykových verzích specifikace stejný.)

<!-- příklad: lflib -->
```json
{
  "format": 1,
  "type": "logicflow-library",
  "app": "LogicFlow 0.4.0",
  "library": {
    "uid": "3f6c2d1a-9b4e-4c7f-8a21-5d0e7b9c4f12",
    "name": "Příklad ze specifikace",
    "description": "Invertor z NAND a opakovač, který ho používá.",
    "translations": { "en": { "name": "Specification example", "description": "An inverter made of NAND and a buffer that uses it." } },
    "author": "Mgr. Stanislav Jan Fidrmuc",
    "license": "CC BY 4.0",
    "version": 1,
    "created": "2026-11-27",
    "modified": "2026-11-27"
  },
  "items": [
    {
      "uid": "b1d7e0c4-2f3a-4e58-9c6d-7a8b9c0d1e2f",
      "id": 1,
      "name": "Invertor z NAND",
      "category": "combinational",
      "description": "Oba vstupy hradla NAND spojené: Y = Ā.",
      "translations": { "en": { "name": "Inverter from NAND", "description": "Both inputs of a NAND gate joined: Y = Ā." } },
      "author": "Mgr. Stanislav Jan Fidrmuc",
      "license": "CC BY 4.0",
      "version": 1,
      "created": "2026-11-27",
      "modified": "2026-11-27",
      "inputs": ["A"],
      "outputs": ["Y"],
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
    },
    {
      "uid": "c2e8f1d5-3a4b-4f69-8d7e-8b9c0d1e2f30",
      "id": 2,
      "name": "Opakovač ze dvou invertorů",
      "category": "combinational",
      "description": "Dva invertory za sebou: Y = A.",
      "translations": { "en": { "name": "Buffer from two inverters", "description": "Two inverters in series: Y = A." } },
      "author": "Mgr. Stanislav Jan Fidrmuc",
      "license": "CC BY 4.0",
      "version": 1,
      "created": "2026-11-27",
      "modified": "2026-11-27",
      "inputs": ["A"],
      "outputs": ["Y"],
      "circuit": {
        "components": [
          { "id": 0, "kind": { "type": "switch", "on": false }, "pos": { "x": 0, "y": 0 }, "rotation": "R0", "label": "A" },
          { "id": 1, "kind": { "type": "subcircuit", "id": 1, "inputs": 1, "outputs": 1 }, "pos": { "x": 8, "y": -1 }, "rotation": "R0", "label": "P1" },
          { "id": 2, "kind": { "type": "subcircuit", "id": 1, "inputs": 1, "outputs": 1 }, "pos": { "x": 18, "y": -1 }, "rotation": "R0", "label": "P2" },
          { "id": 3, "kind": { "type": "led" }, "pos": { "x": 26, "y": 0 }, "rotation": "R0", "label": "Y" }
        ],
        "wires": [
          { "id": 4, "a": { "x": 2, "y": 0 }, "b": { "x": 4, "y": 0 } },
          { "id": 5, "a": { "x": 12, "y": 0 }, "b": { "x": 14, "y": 0 } },
          { "id": 6, "a": { "x": 22, "y": 0 }, "b": { "x": 24, "y": 0 } }
        ],
        "next_id": 7
      }
    }
  ]
}
```
