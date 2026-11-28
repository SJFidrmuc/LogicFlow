# Podobvody

Podobvod je samostatný obvod, který vložíte do jiného jako jednu součástku. Například poloviční sčítačku použijete dvakrát v úplné sčítačce. Vstupy podobvodu jsou jeho přepínače a tlačítka, výstupy jeho LED (s jedním vývodem). Pořadí vývodů na značce určují jejich popisky (A, B, C…).

## Postup

1. V oblasti **Hierarchie** klikněte na **+ Nový podobvod**. Otevře se prázdný list a v záhlaví schématu uvidíte cestu *Hlavní obvod / Podobvod 1*.
2. Nakreslete obvod se vstupy (přepínači) a výstupy (LED) a vyzkoušejte ho, třeba i s pravdivostní tabulkou.
3. V Hierarchii ho pojmenujte v poli **Název**.
4. Klávesou **Tab** se vrátíte do hlavního obvodu. Podobvod pak vložíte z **Shift+A → Podobvody** nebo z Knihovny součástek.

![Úplná sčítačka ze dvou podobvodů a Hierarchie s listy projektu](obrazky/podobvody.png)

## Práce s podobvody

- **Tab** nad vybranou značkou podobvodu ho otevře, další Tab vrátí zpět (jako skupiny uzlů v Blenderu). Mezi listy jde přecházet i kliknutím v Hierarchii.
- Úpravy podobvodu se projeví ve všech jeho instancích. Když přidáte vstup nebo výstup, značka dostane další vývod.
- Podobvod může obsahovat další podobvody, ale ne sám sebe.
- Použitý podobvod nejde smazat. Nejdřív odstraňte jeho instance.
- Hodiny je potřeba přivést zvenku přes vstup podobvodu. Generátor hodin uvnitř podobvodu stojí.
- Simulace zachovává zpoždění hradel i uvnitř podobvodů a každá instance má vlastní stav (např. vlastní klopné obvody).
- Celý projekt včetně všech podobvodů se ukládá do jednoho souboru.

## Podobvod z výběru

Část schématu, kterou už máte nakreslenou, zabalíte do podobvodu přímo na místě:

1. Vyberte hradla a bloky, které mají být uvnitř (tažením myší nebo klikáním se Shiftem). Přepínače a LED klidně vyberte také.
2. **Ctrl+G** (nebo pravé tlačítko → **Vytvořit podobvod z výběru**, případně nabídka Úpravy).

- Vybrané součástky se přesunou do nového podobvodu „Podobvod 1“ (přejmenujete ho v Hierarchii) a na jejich místo se postaví jeho značka.
- **Vstupy a výstupy vzniknou samy** z vodičů, které z výběru vedou ven: co se do výběru přivádí, je vstup, co z něj vychází, je výstup. Značka se k těmto vodičům hned připojí, takže obvod počítá stejně jako předtím.
- **Přepínače, tlačítka, hodiny, LED a zobrazovače zůstanou venku**, i když jsou ve výběru – obvod dál ovládáte a čtete stejně. Vývody podobvodu dostanou jejich popisky (A, B, Y1…), ostatní se pojmenují A, B, C… a Y1, Y2…
- Uvnitř podobvodu jsou přepínače a LED rozhraní na místech, kde vodiče vedly ven – zapojení vypadá jako předtím.
- Když některý vývod nejde připojit bez dotyku s jiným vodičem, program ho nechá volný a upozorní na to. Připojte ho pak ručně.
- **Ctrl+Z** vrátí schéma do stavu před zabalením.

![Hradla AND a OR zabalená do podobvodu – přepínače a LED zůstaly venku](obrazky/podobvod-z-vyberu.png)

## Hotový obvod jako podobvod

Obvod, který už máte uložený v souboru `.logicflow` (třeba vyzkoušenou poloviční sčítačku nebo úlohu z online knihovny obvodů), vložíte do projektu jako podobvod:

1. **Soubor → Importovat jako podobvod…** (**Ctrl+I**) a vyberte soubor. Náhled ukáže název, autora a licenci obvodu.
2. Značka podobvodu se objeví u ukazatele myši – kliknutím ji položíte (Esc vkládání zruší, podobvod v projektu zůstane).

- Podobvod dostane **název obvodu** z jeho vlastností (Soubor → Vlastnosti obvodu), jinak název souboru. Když už projekt podobvod se stejným názvem má, přidá se číslo, např. „Sčítačka (2)“.
- Vstupy jsou přepínače a tlačítka importovaného obvodu, výstupy jeho LED – stejně jako u každého podobvodu.
- Importuje se **kopie**: pozdější úpravy souboru se do projektu nepromítnou a úpravy podobvodu soubor nezmění.
- Podobvody, které importovaný obvod používá, se přidají s ním. Stejné podobvody, které už projekt má, se použijí znovu.
- V režimu simulace import nejde – nejdřív simulaci zastavte (Esc).

## Knihovny prvků

Hotová zapojení najdete v oblasti **Knihovna součástek** pod nadpisem **Knihovny prvků**. Prvky jsou rozdělené podle kategorií (Kombinační obvody, Aritmetika, Sekvenční obvody…) a hledání je najde podle názvu i kategorie.

- **Vestavěná knihovna „Číslicová technika – výukové prvky“** obsahuje 36 zapojení běžných úloh: hradla z NAND a NOR, multiplexory, dekodér, komparátory, převodníky Grayova kódu, sčítačky a odčítačky, klopné obvody, čítače, registry, alarm a semafor.
- **Vestavěná knihovna „Funkční bloky – vnitřní zapojení“** obsahuje zapojení všech funkčních bloků z hradel (viz další část).
- Prvek vložíte stejně jako součástku: klikněte na něj a pak do schématu. Objeví se u ukazatele myši a kliknutím ho položíte.
- Vložený prvek je **podobvod projektu** (kopie). Tabem ho otevřete, upravíte a uloží se s projektem. Prvky, které používá (např. úplné sčítačky ve 4bitové sčítačce), se vloží s ním.
- Pod vybraným prvkem uvidíte jeho popis, vývody, autora a licenci.
- **Osobní knihovny** jsou soubory `.lflib` ve složce `Dokumenty/LogicFlow/Knihovny`. Tlačítko **Otevřít složku** ji vytvoří a otevře, po zkopírování souborů klikněte na **Načíst znovu**. Jinou složku zvolíte v **Úpravy → Nastavení…** (tlačítko **Složky knihoven…**).
- **Sdílené složky** (síť, OneDrive, kroužek) přidáte v **Úpravy → Nastavení… → Knihovny**; jejich knihovny se ukážou s označením **Sdílená složka**.

> **Připravuje se:** export podobvodu do knihovny a školní knihovny.

> **Hotový příklad:** Soubor → Příklady → Úplná sčítačka ze dvou podobvodů.

## Vnitřní zapojení funkčních bloků

Každý funkční blok (multiplexor, dekodér, kodér, sčítačka, komparátor, převodník BCD na 7 segmentů, čítač, registr, posuvný registr) jde rozkliknout až na hradla.

- Vyberte blok ve schématu a stiskněte **Tab** (nebo pravé tlačítko → **Otevřít vnitřní zapojení**). Otevře se jeho zapojení z hradel se stejně pojmenovanými vývody.
- Zapojení je **jen pro čtení**: vlevo nahoře je štítek s názvem bloku, vstupy jde přepínat a simulovat, ale schéma nejde měnit. Projekt zůstane beze změny.
- Sekvenční bloky jsou z klopných obvodů T a D. **Tab** na klopném obvodu jde hlouběji – klopný obvod D je ze šesti hradel NAND (zapojení obvodu 7474).
- **Tab** v prázdném místě vrátí o úroveň výš, z nejvyšší úrovně zpět do projektu. Rovnou do projektu vede tlačítko **Zpět do projektu** v Hierarchii nebo odkaz **Hlavní obvod** v záhlaví schématu.
- Tlačítko **Vložit do projektu jako podobvod** v Hierarchii zkopíruje zapojení do projektu jako upravitelný podobvod (vloží se u ukazatele myši jako prvek z knihovny).
- Zapojení z hradel se od bloku liší ve dvou věcech: nepřipojené vstupy **EN** a **R** mají hodnotu X (blok bere EN jako 1 a R jako 0) a sekvenční zapojení je po spuštění v neznámém stavu X, dokud ho **R = 1** nevynuluje (blok začíná ve stavu 0).
- Všechna zapojení najdete i v panelu knihoven v knihovně **Funkční bloky – vnitřní zapojení**.
