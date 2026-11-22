# Změny a plán vývoje LogicFlow

**Česky** · [English](CHANGELOG.en.md)

Přehled toho, co přinesla která verze a co se chystá do dalších. Nejnovější verze je nahoře, plán dalších verzí je na konci.

- **Nevydáno** – změny už sloučené do `main` (nebo připravené v pull requestu), které vyjdou v příští verzi. Každý pull request, který mění něco viditelného nebo důležitého, sem připíše řádek česky i anglicky.
- **Při vydání verze** se sekce Nevydáno přejmenuje na číslo verze s datem a založí se nová prázdná. Plán dalších verzí se upraví podle skutečnosti. Postup je v [CONTRIBUTING.md](CONTRIBUTING.md) → Vydání nové verze; dodržení hlídá `python3 scripts/zmeny.py kontrola` v CI i při vydání.

Hotové programy ke stažení: [Releases](https://github.com/SJFidrmuc/LogicFlow/releases).

## [Nevydáno]

### Přidáno
- **Napájení VCC a zem GND** se značkami podle elektrotechnických schémat (logická 1 a 0, bez automatického popisku). Spojení VCC s GND je **zkrat napájení**: síť zčervená a schéma i stavový řádek ukážou hlášku, i když je zkrat uvnitř podobvodu.
- **Průběžné sestavení z větve main:** předběžné vydání „Průběžné sestavení (main)“ s balíčky pro všechny platformy se přepíše po každé změně – novinky se dají vyzkoušet bez čekání na vydání. **Archiv verzí** na webu: tabulka všech vydání × platforem s odkazy ke stažení.
- **Jemnější nastavení hodin:** frekvence posuvníkem 0,1–100 Hz (logaritmicky, s polem pro zadání čísla), periodou v ms nebo rychlými volbami; změna i za běhu simulace bez skoku fáze – hodiny navážou plynule.
- **Rychlost simulace ×0,1 až ×10** ve stavovém řádku a v nabídce Simulace; stavový řádek ukazuje simulovaný čas.
- **Krok na další hranu hodin** (Shift+šipka vpravo) a **krok o celou periodu** na další náběžnou hranu (Ctrl+šipka vpravo).
- **Schránka Ctrl+C / X / V**: kopírování vybraných součástek i s vodiči mezi nimi, vložení k ukazateli myši (vložená část se hned posouvá, klik ji položí); funguje mezi listy i mezi dvěma okny LogicFlow, použité podobvody se přenesou s sebou. Položky Kopírovat, Vyjmout a Vložit i v nabídce Úpravy a v kontextové nabídce.
- **Kontextová nabídka na pravém tlačítku** podle místa kliknutí: u součástky Přepnout, Frekvence, Počet vstupů, Strana vývodů, Počet bitů, Směr čítání, Otevřít podobvod, popisek, Otočit, Duplikovat, Přesunout, Smazat; u vodiče Vybrat / Smazat celou síť a Smazat úsek; v prázdném místě Přidat součástku sem, Vybrat vše, Celé schéma a ovládání simulace. Tažení pravým tlačítkem dál posouvá pohled. V režimu simulace jsou úpravy zašedlé.
- Pravé tlačítko i v Hierarchii (Otevřít a Smazat podobvod, Najít ve schématu) a v pravdivostní tabulce (Kopírovat jako text – do Wordu nebo tabulky po sloupcích).
- **Režim úprav a režim simulace** (jako v DigitalWorks): F5 spustí simulaci a schéma se zamkne – fungují jen přepínače, tlačítka, posun, zoom, Tab a frekvence hodin; Esc nebo Shift+F5 vrátí k úpravám a stav obvodu (klopné obvody, čítače, LED, časový diagram) zůstane. Jantarový rámeček se štítkem SIMULACE, režim ve stavovém řádku, hláška „Pro úpravy zastavte simulaci (Esc)“. Volba Simulace → Povolit úpravy za běhu simulace (výchozí vypnuto).
- Pracovní plocha **Simulace** zapíná režim simulace: přepnutím na ni se simulace spustí a schéma zamkne, odchodem z ní (nebo Esc) se zastaví a program se vrátí na předchozí plochu.
- Nabídky ukazují u každé položky klávesovou zkratku a nedostupné položky jsou zašedlé; nabídka Úpravy nově nabízí Vybrat vše, Zrušit výběr, Otočit, Smazat a Otevřít podobvod / zpět, nabídka Zobrazit přepínání pracovních ploch.
- Jednotný seznam příkazů v programu – základ pro kontextovou nabídku a panel nástrojů.
- Ctrl+Y jako druhá zkratka pro Znovu.
- Přehled změn a plánu vývoje v tomto souboru (česky a anglicky) – kontrola v CI, poznámky k vydání se z něj doplní samy.
- Komentáře ve zdrojovém kódu česky i anglicky ve všech souborech; kontrola v CI a export kopie zdrojů s komentáři jen v jednom jazyce (`scripts/komentare.py`).
- Formát knihoven prvků `.lflib` v jádře (načtení, uložení, kontrola, názvy ve více jazycích, kategorie) – základ pro knihovny ve verzi 0.4; popis formátu a vzorová knihovna v `docs/knihovny`.
- **Soubor s nastavením `logicflow.toml`** v domovské složce (Windows `%APPDATA%\LogicFlow` – s cestovním profilem se přenáší, macOS `~/Library/Application Support/LogicFlow`, Linux `~/.config/logicflow`): čitelný text s komentáři, bezpečný zápis, nečitelný soubor se odloží jako `.bak`. Dosavadní nastavení jazyka a značek se převezme. Okno **Úpravy → Nastavení…**: jazyk, značky, úpravy za běhu simulace, jméno autora, výchozí licence, složka osobních knihoven, **sdílené složky knihoven** (síť, OneDrive, kroužek; volba jen pro čtení), otevření složky s nastavením a obnovení výchozích hodnot. Složka Dokumenty se hledá přes systémové známé složky (funguje i přesměrování na síť).
- **Knihovny prvků v Knihovně součástek:** vestavěná výuková knihovna „Číslicová technika“ a osobní knihovny ze složky `Dokumenty/LogicFlow/Knihovny` (tlačítka Otevřít složku a Načíst znovu), prvky podle kategorií, hledání podle názvu i kategorie, u vybraného prvku popis, vývody, autor a licence. Prvek se vloží jako podobvod projektu (kopie) i s prvky, které používá – stejně jako Ctrl+V u ukazatele myši; názvy v jazyce rozhraní. Vadný soubor knihovny se ukáže jako chyba, program nespadne.
- **Výuková knihovna „Číslicová technika“** (`docs/knihovny/cislicova-technika.lflib`, v programu jako vestavěná knihovna): 36 hradlových zapojení běžných úloh – hradla z NAND a NOR, multiplexory, dekodér, komparátory, převodníky Grayova kódu, parita, sčítačky a odčítačky, klopné obvody RS a D, čítače (asynchronní, synchronní, BCD, nahoru/dolů), posuvný registr, Johnsonův a kruhový čítač, alarm, indikace ventilátorů a semafor; česky i anglicky, každý prvek ověřený simulací.

### Opraveno
- Ctrl+R restartuje simulaci (zkratka byla v nabídce a nápovědě, ale nefungovala).
- Klávesy nad schématem reagují jen na přesnou kombinaci (Shift+X už nemaže jako X).
- Knihovna součástek a editor paměti uvádějí u připravovaných součástek aktuální plán (paměti ve verzi 0.5, obvody 74xx ve verzi 0.7).

### Změněno
- **Plán verzí rozdělen:** 0.4 bez sběrnic a edice Škola, ty přijdou ve 0.5; další milníky se posunuly o jednu verzi. Připravované součástky v knihovně a panel Paměť ukazují nové verze (paměti 0.6, obvody 74xx 0.8).
- Jméno autora se všude uvádí s titulem – Mgr. Stanislav Jan Fidrmuc (okno O aplikaci, licence, nápověda, knihovny, README); hlídá to test.
- V režimu úprav se přepínač přepíná dvojklikem a tlačítko stiskne druhým stiskem dvojkliku – jedno kliknutí součástku jen vybere, takže ji při přesouvání nepřepnete omylem. Hodiny v režimu úprav stojí.
- Mezerník v režimu úprav spustí simulaci, šipka vpravo z režimu úprav přejde do pozastavené simulace a udělá krok.
- Příprava na budoucí webovou verzi (iPhone, iPad, Android, Chromebook): simulační jádro bez práce se soubory a systémového času, překládá se v CI i pro WebAssembly; čas v aplikaci přes `web_time`. Chování programu se nemění.
- Workflow vydání před sestavením ověří, že tag odpovídá verzi programu a že je verze popsaná v přehledu změn.

## [0.3.1] – 2026-11-21

### Přidáno
- Nápověda se píše v Markdownu (`docs/napoveda`, česky i anglicky) a ke každému vydání se připojí jako HTML (ZIP), volitelně i na web (GitHub Pages).
- Jazyk (čeština / angličtina) a styl značek (evropské / americké) se při prvním spuštění zvolí podle systému.
- Odkaz na web projektu [www.logicflow-sim.cz](https://www.logicflow-sim.cz) v nabídce Nápověda a v okně O aplikaci.
- Diagram struktury aplikace a toku dat (`docs/architektura.svg`).

### Opraveno
- Hodiny tikají rovnoměrně podle skutečného času (dříve v klidu běžely asi třikrát pomaleji).

### Změněno
- Akce GitHubu v CI a ve vydání na Node.js 24.

## [0.3.0] – 2026-11-20

Bez samostatného vydání na GitHubu – obsah vyšel ve verzi 0.3.1.

### Přidáno
- **Funkční bloky:** multiplexor, dekodér / demultiplexor, prioritní kodér, sčítačka, komparátor, převodník BCD na 7 segmentů, čítač nahoru / dolů, registr a posuvný registr s nastavitelným počtem bitů; obdélníkové značky podle ČSN EN 60617-12.
- **Zobrazovače:** sedmisegmentovka a hex displej, volba strany vývodů.
- **Podobvody (makra):** listy projektu, rozhraní z přepínačů a LED, Tab do podobvodu a zpět, vnoření až do hloubky 16; podobvody se ukládají do souboru obvodu.
- **Americké značky** (ANSI/IEEE Std 91) přepínatelné v nabídce Zobrazit; nastavení a velikost okna se pamatují mezi spuštěními.
- Okno pro soubory: tlačítko Nová složka.
- Příklady: multiplexor 4:1, dekadický čítač se sedmisegmentovkou, úplná sčítačka z podobvodů.

### Opraveno
- Posun součástek a kreslení vodičů už nevytváří nechtěné spoje; náhled vodiče označí místa napojení.

## [0.2.0] – 2026-11-19

### Přidáno
- **Sekvenční logika:** klopné obvody RS, D řízený úrovní, D, JK a T řízené hranou (náběžnou i sestupnou), počáteční stav; generátor hodin s nastavitelnou frekvencí.
- Restart simulace (Ctrl+R), hodiny v časovém diagramu, nabídka Shift+A s kategoriemi.
- **Nápověda** (F1) česky i anglicky jako okno i jako editor v oblasti; rozdělování oblastí (vedle sebe, pod sebe).
- Automatické balíčky pro šest platforem na stránce Releases (Windows x64/ARM64, macOS ARM/Intel, Linux x64, Raspberry Pi).
- Příklady: klopný obvod D s hodinami, čítač 0–7, posuvný registr.

### Změněno
- Negace se kreslí indikátorem negace (trojúhelník) podle ČSN EN 60617-12.

## [0.1.0] – 2026-11-18

Bez vydání na GitHubu (jen zdrojový kód).

### Přidáno
- **Kreslení schémat:** součástky na mřížce s otáčením, vodiče tažením, odbočky, výběr obdélníkem, přesun, duplikace, mazání, zpět / znovu.
- **Simulace:** událostní simulátor se zpožděním hradel, hodnotami 0 / 1 / X / Z, krokováním a odhalením kmitání.
- Součástky: přepínač, tlačítko, konstanty, hradla se 2–8 vstupy, LED.
- Panel vlastností, hierarchie, časový diagram a pravdivostní tabulka ze skutečného obvodu.
- Ukládání a otevírání souborů `.logicflow`, vlastní okno pro soubory, dotaz na neuložené změny.
- Příklady: AND / OR, poloviční sčítačka, klopný obvod RS z hradel NOR.

## [0.0.1] – 2026-11-18

Základ projektu, bez vydání.

### Přidáno
- Rozhraní ve stylu Blenderu: tmavý motiv, dělitelné oblasti, pracovní plochy Návrh / Simulace / Analýza / Paměť.
- Simulační jádro s hodnotami 0 / 1 / X / Z a základními hradly, čeština a angličtina.
- CI pro Windows, macOS, Linux a Raspberry Pi.

## Plán dalších verzí

Orientační – pořadí a obsah se mohou změnit. Podrobný plán vede autor projektu.

### 0.4 (2027)
- **Režim úprav a režim simulace:** za běhu simulace je schéma zamčené, fungují jen ovládací prvky; stav obvodu po zastavení zůstává.
- **Kontextová nabídka** na pravém tlačítku myši a **schránka** (Ctrl+C / X / V).
- Jemnější nastavení frekvence hodin a rychlosti simulace, krokování po hraně.
- Napájení **VCC a GND** se značkami.
- Autor, licence a popis přímo v souboru obvodu.
- Barva LED a LED se dvěma vývody (anoda, katoda), import hotového obvodu jako podobvodu.
- Soubor s nastavením v domovské složce.
- Jazykové balíčky, nápověda v programu přímo z Markdownu.
- Světlý motiv (projektor, tisk), export obrázku schématu PNG / SVG.
- Kontrola aktualizací, úvodní obrazovka, odkaz Podpořit vývoj.
- **Knihovny prvků:** vestavěná výuková knihovna, osobní knihovny a sdílené složky, export podobvodu do knihovny.

### 0.5 (2027)
- **Sběrnice 1–64 bitů:** rozdělovač, třístavový budič, vstup s číslem, hex klávesnice, logická sonda, funkční bloky se sběrnicovými vývody.
- **Edice Škola:** nastavení pro učebnu se zamykáním, Správa učebny, školní knihovny (učitelská a výměnná složka třídy).

### 0.6 (2027)
- Paměti ROM, RAM, EEPROM s hex editorem.
- Panel nástrojů s ikonami ve dvou podobách: kompaktní a velké ikony se záložkami (dotyk, projektor).
- Plošky rozhraní podobvodu (vstupy a výstupy podobvodu s názvem a pořadím, i sběrnicové) a návěští sítí – propojení podle názvu bez vodičů, i pro VCC a GND.
- **Maticová zapojení:** maticová klávesnice 4×4 a 3×4, bodová matice LED 5×7 a 8×8 s dosvitem (skenování a multiplexování displejů).

### 0.7 (2027)
- Export do VHDL.

### 0.8 (2027)
- Obvody řady 74xx (pouzdro DIP s vývody VCC a GND), příručka v PDF, vzorové úlohy.
- Tlačítko a spínač se dvěma vývody, rezistor pull-up, výstupy s otevřeným kolektorem.

### 1.0 (září 2027)
- Verze pro výuku.

### Kolem verze 1.0 a později
- **Webová verze** pro iPhone, iPad, Android a Chromebooky (v prohlížeči, instalovatelná na plochu) s dotykovým ovládáním.
- Online knihovna obvodů přímo v programu.
- Verilog, Karnaughovy mapy, nahrání do FPGA, úlohy s automatickou kontrolou.

[Nevydáno]: https://github.com/SJFidrmuc/LogicFlow/compare/v0.3.1...HEAD
[0.3.1]: https://github.com/SJFidrmuc/LogicFlow/releases/tag/v0.3.1
[0.3.0]: https://github.com/SJFidrmuc/LogicFlow/compare/v0.2.0...v0.3.1
[0.2.0]: https://github.com/SJFidrmuc/LogicFlow/releases/tag/v0.2.0
[0.1.0]: https://github.com/SJFidrmuc/LogicFlow/commit/2d78684
[0.0.1]: https://github.com/SJFidrmuc/LogicFlow/commit/78db4ae
