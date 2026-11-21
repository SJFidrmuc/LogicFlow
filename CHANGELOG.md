# Změny a plán vývoje LogicFlow

**Česky** · [English](CHANGELOG.en.md)

Přehled toho, co přinesla která verze a co se chystá do dalších. Nejnovější verze je nahoře, plán dalších verzí je na konci.

- **Nevydáno** – změny už sloučené do `main` (nebo připravené v pull requestu), které vyjdou v příští verzi. Každý pull request, který mění něco viditelného nebo důležitého, sem připíše řádek česky i anglicky.
- **Při vydání verze** se sekce Nevydáno přejmenuje na číslo verze s datem a založí se nová prázdná. Plán dalších verzí se upraví podle skutečnosti. Postup je v [CONTRIBUTING.md](CONTRIBUTING.md) → Vydání nové verze; dodržení hlídá `python3 scripts/zmeny.py kontrola` v CI i při vydání.

Hotové programy ke stažení: [Releases](https://github.com/SJFidrmuc/LogicFlow/releases).

## [Nevydáno]

### Přidáno
- **Režim úprav a režim simulace** (jako v DigitalWorks): F5 spustí simulaci a schéma se zamkne – fungují jen přepínače, tlačítka, posun, zoom, Tab a frekvence hodin; Esc nebo Shift+F5 vrátí k úpravám a stav obvodu (klopné obvody, čítače, LED, časový diagram) zůstane. Jantarový rámeček se štítkem SIMULACE, režim ve stavovém řádku, hláška „Pro úpravy zastavte simulaci (Esc)“. Volba Simulace → Povolit úpravy za běhu simulace (výchozí vypnuto).
- Pracovní plocha **Simulace** zapíná režim simulace: přepnutím na ni se simulace spustí a schéma zamkne, odchodem z ní (nebo Esc) se zastaví a program se vrátí na předchozí plochu.
- Nabídky ukazují u každé položky klávesovou zkratku a nedostupné položky jsou zašedlé; nabídka Úpravy nově nabízí Vybrat vše, Zrušit výběr, Otočit, Smazat a Otevřít podobvod / zpět, nabídka Zobrazit přepínání pracovních ploch.
- Jednotný seznam příkazů v programu – základ pro kontextovou nabídku a panel nástrojů.
- Ctrl+Y jako druhá zkratka pro Znovu.
- Přehled změn a plánu vývoje v tomto souboru (česky a anglicky) – kontrola v CI, poznámky k vydání se z něj doplní samy.
- Komentáře ve zdrojovém kódu česky i anglicky ve všech souborech; kontrola v CI a export kopie zdrojů s komentáři jen v jednom jazyce (`scripts/komentare.py`).

### Opraveno
- Ctrl+R restartuje simulaci (zkratka byla v nabídce a nápovědě, ale nefungovala).
- Klávesy nad schématem reagují jen na přesnou kombinaci (Shift+X už nemaže jako X).
- Knihovna součástek a editor paměti uvádějí u připravovaných součástek aktuální plán (paměti ve verzi 0.5, obvody 74xx ve verzi 0.7).

### Změněno
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
- Barva LED, plošky rozhraní podobvodu, import hotového obvodu jako podobvodu.
- Soubor s nastavením v domovské složce, **edice Škola** (nastavení pro učebnu, Správa učebny).
- Jazykové balíčky, nápověda v programu přímo z Markdownu.
- Světlý motiv (projektor, tisk), export obrázku schématu PNG / SVG.
- Kontrola aktualizací, úvodní obrazovka, odkaz Podpořit vývoj.
- Knihovny bloků (osobní, sdílená složka, školní).
- **Sběrnice 1–64 bitů:** rozdělovač, třístavový budič, hex klávesnice, logická sonda.

### 0.5 (2027)
- Paměti ROM, RAM, EEPROM s hex editorem.
- Panel nástrojů s ikonami ve dvou podobách: kompaktní a velké ikony se záložkami (dotyk, projektor).

### 0.6 (2027)
- Export do VHDL.

### 0.7 (2027)
- Obvody řady 74xx (pouzdro DIP s vývody VCC a GND), příručka v PDF, vzorové úlohy.

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
