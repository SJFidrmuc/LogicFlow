# Kreslení schématu

## Přidání součástky

- **Shift+A** nad schématem otevře nabídku Přidat rozdělenou do skupin: Vstupy, Hradla, Klopné obvody, Funkční bloky, Výstupy a Podobvody.
- Nebo klikněte na součástku v **Knihovně součástek** a pak do schématu. Pole Hledat nahoře v knihovně filtruje podle názvu.
- Při umisťování klávesa **R** otáčí o 90°, **Shift+klik** umístí další kus stejné součástky, **Esc** nebo pravé tlačítko umisťování zruší.

![Nabídka Shift+A se skupinami součástek](obrazky/shift-a.png)

## Vodiče

- Vodič začíná tažením z vývodu součástky (vývod se pod myší zvýrazní kroužkem) nebo z libovolného bodu existujícího vodiče. S klávesou Ctrl jde začít kdekoli.
- Vodič se láme do pravého úhlu. Směr prvního úseku určí první pohyb myši.
- Kde konec vodiče leží na jiném vodiči, vznikne spoj a zobrazí se tečka. Vodiče, které se jen kříží (bez tečky), propojené nejsou.
- Vývod součástky se připojí jen ke konci vodiče. Vodič, který přes vývod jen prochází, se s ním nespojí.
- Při kreslení oranžový kroužek ukazuje místo, kde by se vodič napojil i jinde než na začátku a konci. Pokud má opačné zalomení méně takových míst, program ho použije sám.
- Při přesunu součástky se připojené vodiče zalomí tak, aby se nedotkly jiného spoje. Když to nejde, vodič se od vývodu odpojí: volný konec je vidět a stačí ho dotáhnout.

![Spoj s tečkou a křížení vodičů bez spojení](obrazky/vodice.png)

## Výběr a úpravy

| Akce | Co udělá |
| --- | --- |
| klik | vybrat součástku nebo vodič |
| Shift+klik | přidat do výběru nebo odebrat z výběru |
| tažení v prázdném místě | výběr obdélníkem |
| tažení součástky, G | přesun výběru (vodiče se přepojí) |
| R | otočit o 90° |
| X, Delete | smazat výběr |
| Shift+D | duplikovat výběr |
| Ctrl+C, Ctrl+X | kopírovat, vyjmout výběr do schránky |
| Ctrl+V | vložit ze schránky k ukazateli myši |
| A, Alt+A | vybrat vše, zrušit výběr |
| Ctrl+Z, Ctrl+Shift+Z | zpět, znovu |

### Schránka

- **Ctrl+C** zkopíruje vybrané součástky i s vodiči, které je propojují (vodiče k nevybraným součástkám se nezkopírují). **Ctrl+X** výběr zároveň smaže.
- **Ctrl+V** vloží obsah schránky k ukazateli myši; vložená část se hned posouvá s myší a kliknutím ji položíte (Esc ji nechá na místě vložení).
- Kopírovat a vkládat jde mezi listy (hlavní obvod, podobvody) i mezi dvěma spuštěnými okny LogicFlow. Použité podobvody se přenesou s sebou; stejné se znovu nepřidávají, při shodě názvů dostane nový podobvod číslo, např. „Sčítačka (2)“.
- Popisky (A, B, Y1…) zůstanou, pokud v cílovém obvodu ještě nejsou; jinak dostanou nové.
- Schránka obsahuje obvod jako text ve formátu souboru .logicflow – stejně jde vložit i celý obsah souboru otevřeného v textovém editoru.

## Kontextová nabídka (pravé tlačítko)

**Kliknutí pravým tlačítkem** (bez tažení) otevře nabídku podle toho, kam kliknete. Tažení pravým tlačítkem dál posouvá pohled a během rozdělané akce (přesun, kreslení vodiče, umisťování) pravé tlačítko akci zruší.

| Kam kliknete | Co nabídka nabízí |
| --- | --- |
| součástka | podle druhu: Přepnout (přepínač), Frekvence (hodiny), Počet vstupů (hradlo), Strana vývodů (zobrazovač), Barva a Vývody (LED), Počet bitů a Směr čítání (funkční blok), Otevřít podobvod; dále popisek, Otočit, Duplikovat, Přesunout, Kopírovat, Vyjmout, Smazat, Vytvořit podobvod z výběru |
| vodič | Vybrat celou síť, Smazat úsek vodiče, Smazat celou síť |
| výběr více prvků | Otočit, Duplikovat, Přesunout, Kopírovat, Vyjmout, Smazat, Zrušit výběr, Vytvořit podobvod z výběru |
| prázdné místo | Přidat součástku sem (vloží se na místo kliknutí), Vložit, Vybrat vše, Celé schéma, Zpět o úroveň výš (v podobvodu), ovládání simulace |

- Nevybraná součástka nebo vodič se kliknutím pravým tlačítkem nejdřív vybere.
- U každé položky je vpravo klávesová zkratka – nabídka tak zkratky i učí.
- V režimu simulace jsou úpravy zašedlé; funguje Přepnout, Frekvence hodin, Otevřít podobvod, výběr a ovládání simulace.
- Pravé tlačítko funguje i v **Hierarchii** (u listu: Otevřít, Smazat podobvod; u součástky: Najít ve schématu, Přepnout, Otočit, Smazat) a v **pravdivostní tabulce** (Kopírovat jako text).

## Pohled na schéma

- **Kolečko myši** přibližuje a oddaluje, **prostřední nebo pravé tlačítko** s tažením posouvá pohled (samotné kliknutí pravým tlačítkem otevře kontextovou nabídku).
- **Home** nebo tlačítko **Celé schéma** v záhlaví zobrazí celý obvod. Vedle tlačítka je vidět aktuální zvětšení v procentech.
- Dole ve schématu je řádek s nejdůležitějšími klávesami.

## Vlastnosti součástky

V oblasti **Vlastnosti** se u vybrané součástky zobrazí její typ a nastavení. Podle druhu součástky jde změnit:

- **popisek** (např. A, B, CLK, Y1); určuje i pořadí signálů v časovém diagramu a pravdivostní tabulce,
- **druh hradla** a **počet vstupů** (2–8),
- **hodnotu konstanty**, **frekvenci** generátoru hodin,
- u klopných obvodů **druh**, **spouštění** (úroveň, náběžná nebo sestupná hrana) a **počáteční stav**,
- u funkčních bloků **počet bitů** a u čítače **směr čítání**,
- u zobrazovačů **stranu vývodů**,
- **otočení**; tlačítko **Smazat** součástku odstraní.

Vlastnosti zároveň ukazují **živé hodnoty** součástky: stav, vstupy a výstupy (u čítače např. 0011 = 3) a zpoždění v taktech.

![Oblast Vlastnosti s vybraným klopným obvodem JK](obrazky/vlastnosti.png)
