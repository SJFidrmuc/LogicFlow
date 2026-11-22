# Knihovny prvků – formát `.lflib`

**Česky** · [English](#english)

Knihovna je soubor hotových podobvodů – **prvků** –, které jde vložit do libovolného projektu (sčítačky, převodníky kódů, čítače, vlastní bloky). Program knihovny zatím neumí otevřít; formát je připravený pro verzi 0.4 (etapa K1 – panel knihoven, vložení prvku, export podobvodu do knihovny). V jádře ho čte a zapisuje modul `crates/logicflow-core/src/library.rs`.

Vzor: [`vzor.lflib`](vzor.lflib) – úplná sčítačka 1 bit a 2bitová sčítačka, která ji dvakrát používá. Test v jádře ověřuje, že se vzor načte, že oba prvky správně sčítají (všechny kombinace vstupů) a že ho program uloží beze změny.

## Výuková knihovna „Číslicová technika“

[`cislicova-technika.lflib`](cislicova-technika.lflib) – 36 hradlových zapojení běžných úloh číslicové techniky (podle osnov a sbírky úloh rc2000 – µLAB, kap. 7). Každý prvek má český i anglický název a popis, licenci CC BY 4.0 a je ověřený simulací.

| Kategorie | Prvky |
| --- | --- |
| Kombinační obvody | NOT, AND, OR z NAND; XOR ze čtyř NAND; NOT, OR, AND z NOR; XNOR ze čtyř NOR; majoritní obvod 2 ze 3; generátor sudé parity 4 bity; multiplexor 2:1 a 4:1 z hradel; dekodér 2 → 4 z hradel; komparátor 1 bit; komparátor 2 bity (ze dvou komparátorů 1 bit) |
| Převodníky kódů | binární → Grayův kód 4 bity; Grayův kód → binární 4 bity |
| Aritmetika | poloviční sčítačka; úplná sčítačka ze dvou polovičních; sčítačka 4 bity (ze čtyř úplných); poloviční a úplná odčítačka; sčítačka a odčítačka 4 bity |
| Sekvenční obvody | klopný obvod RS z NOR a z NAND; D (úroveň) z NAND; asynchronní čítač 4 bity (T); synchronní čítač 4 bity (JK); synchronní čítač modulo 10 (BCD); čítač nahoru / dolů 3 bity; posuvný registr 4 bity; Johnsonův a kruhový čítač 4 bity |
| Aplikace | alarm s pamětí; indikace funkce ventilátorů; semafor |

Rozhraní prvků tvoří přepínače (vstupy) a LED (výstupy) uvnitř; u sekvenčních obvodů jsou hodiny vstup **C**. Prvky, které používají jiné prvky (úplná sčítačka, sčítačka 4 bity, komparátor 2 bity, sčítačka a odčítačka), se vloží i s nimi.

Knihovnu **generuje test** `crates/logicflow-core/tests/vyukova_knihovna.rs`: zapojení je popsané jako netlist a sestavovač schémat (`tests/sestavovac`) ho rozmístí do sloupců a propojí vodiči; z hotového schématu pak znovu postaví netlist a porovná ho se zamýšleným zapojením. Testy projdou pravdivostní tabulky všech kombinačních prvků, průběh čítačů a registrů a chování klopných obvodů a aplikací. Po změně prvků:

```sh
LOGICFLOW_ZAPSAT_KNIHOVNU=1 cargo test -p logicflow-core --test vyukova_knihovna
```

Dokud program knihovny neumí otevřít, lze prvky prohlédnout jako projekty: `LOGICFLOW_NAHLED=<složka> cargo test -p logicflow-core --test vyukova_knihovna nahled` zapíše každý prvek jako samostatný `.logicflow` a katalog `cislicova-technika.logicflow` se všemi prvky jako podobvody.

## Stavba souboru

JSON ve stejném stylu jako projekt `.logicflow`:

```json
{
  "format": 1,
  "type": "logicflow-library",
  "app": "LogicFlow 0.4.0",
  "library": {
    "uid": "8a3e6f52-1c4d-4b9e-a7f0-3d2c5b6e9f81",
    "name": "Vzorová knihovna",
    "description": "…",
    "translations": { "en": { "name": "Sample library", "description": "…" } },
    "author": "Stanislav Jan Fidrmuc",
    "license": "CC BY 4.0",
    "version": 1,
    "created": "2026-11-22",
    "modified": "2026-11-22"
  },
  "items": [
    {
      "uid": "ce146411-078a-4bdc-8ad0-5b80bbab6fa4",
      "id": 1,
      "name": "Úplná sčítačka 1 bit",
      "category": "arithmetic",
      "description": "…",
      "translations": { "en": { "name": "1-bit full adder", "description": "…" } },
      "author": "Stanislav Jan Fidrmuc",
      "license": "CC BY 4.0",
      "version": 1,
      "created": "2026-11-22",
      "modified": "2026-11-22",
      "inputs": ["A", "B", "Cin"],
      "outputs": ["Cout", "S"],
      "circuit": { "components": [ … ], "wires": [ … ] }
    }
  ]
}
```

| Pole | Význam |
| --- | --- |
| `format` | verze formátu (teď 1); soubor z novější verze program odmítne s vysvětlením |
| `type` | vždy `logicflow-library` – podle něj se knihovna pozná od projektu |
| `uid` | **stabilní identita** knihovny i prvku: UUID, nemění se při přejmenování ani úpravě; podle ní se pozná, ze kterého prvku vznikla kopie v projektu |
| `id` | číslo prvku v souboru; na něj odkazují instance prvku uvnitř jiných prvků téže knihovny (`"type": "subcircuit", "id": 1`) |
| `name`, `description` | název a popis **v jazyce autora** |
| `translations` | nepovinné překlady podle kódu jazyka ISO 639-1 (`en`, `cs`, `sk`, `uk` …); program ukáže název v jazyce rozhraní, jinak hlavní |
| `category` | vestavěná kategorie jako klíč (tabulka níže) nebo vlastní text |
| `author`, `license` | autor a licence; výchozí licence nových prvků je **CC BY 4.0** (stejně jako online knihovna obvodů) |
| `version` | zvyšuje se při každé úpravě prvku (u knihovny při každém uložení) |
| `created`, `modified` | datum RRRR-MM-DD |
| `inputs`, `outputs` | názvy vývodů – jen pro čtenáře souboru a web knihovny; program je při uložení přepočítá ze zapojení (popisky přepínačů a LED, seřazené přirozeně) a při načtení nepoužije |
| `circuit` | zapojení prvku – stejná struktura jako `circuit` v `.logicflow` |

**Kontrola při načtení:** platná UUID, žádné dvě stejné identity ani čísla, každá použitá instance odkazuje na prvek v knihovně a žádný prvek neobsahuje (ani nepřímo) sám sebe.

## Vestavěné kategorie

| Klíč | Česky | English |
| --- | --- | --- |
| `combinational` | Kombinační obvody | Combinational circuits |
| `arithmetic` | Aritmetika | Arithmetic |
| `code-converters` | Převodníky kódů | Code converters |
| `sequential` | Sekvenční obvody | Sequential circuits |
| `memory` | Paměti | Memories |
| `applications` | Aplikace | Applications |
| `other` | Ostatní | Other |

Při uložení se český nebo anglický název vestavěné kategorie převede na klíč (`Aritmetika` → `arithmetic`), takže se v panelu zobrazí v jazyce rozhraní. Vlastní kategorie se uloží tak, jak ji autor napsal.

## Vložení prvku do projektu

Prvek se do projektu vkládá **jako kopie** – stejně jako Ctrl+V: jádro připraví text ve tvaru schránky (`Library::insertion_text`) s jednou instancí prvku a definicemi všech prvků, které používá, a projekt ho vloží stejnou cestou jako ze schránky (stejné podobvody se použijí znovu, kolize názvů se přejmenují). Záznam původu (knihovna, uid a verze prvku) přibude v dalším kroku K1.

## Pro vývojáře

- UUID nového prvku vytvoří aplikace: `library::uid_from_random(bajty)` z 16 náhodných bajtů. Jádro samo žádný zdroj náhody ani systémový čas nemá (kvůli webové verzi), datum dodává aplikace.
- Chyby (`LibraryError`) mají text česky i anglicky (`LibraryError::text(jazyk)`).

---

## English

A library is a file of ready-made subcircuits – **items** – that can be inserted into any project. The program cannot open libraries yet; the format is prepared for version 0.4 (stage K1). The core reads and writes it in `crates/logicflow-core/src/library.rs`; [`vzor.lflib`](vzor.lflib) is a sample (a 1-bit full adder and a 2-bit adder that uses it twice).

- The file is JSON like `.logicflow`, with `"type": "logicflow-library"`, a `library` header and a list of `items`.
- `uid` is the **stable identity** (a UUID) of the library and of each item; `id` is the item number that instances inside other items of the same library refer to.
- `name` and `description` are in the author's language, `translations` holds optional translations by ISO 639-1 code; the program shows the name in the interface language when available.
- `category` is the key of a built-in category (table above) or custom text. The default licence of new items is **CC BY 4.0**.
- `inputs` / `outputs` are informative only; they are recomputed from the circuit on saving.
- On loading the core checks valid and unique UUIDs, unique numbers, references between items and that no item contains itself.
- An item is inserted into a project **as a copy**, the same way as Ctrl+V (`Library::insertion_text`).

**Teaching library “Digital electronics”** – [`cislicova-technika.lflib`](cislicova-technika.lflib): 36 gate-level circuits of common tasks (gates from NAND/NOR, multiplexers, decoder, comparators, Gray code converters, parity, adders and subtractors, RS/D flip-flops, asynchronous, synchronous, BCD and up/down counters, shift register, Johnson and ring counters, alarm, fan indicator, traffic light), with Czech and English names. It is generated and verified by simulation in `crates/logicflow-core/tests/vyukova_knihovna.rs`; regenerate with `LOGICFLOW_ZAPSAT_KNIHOVNU=1 cargo test -p logicflow-core --test vyukova_knihovna`.
