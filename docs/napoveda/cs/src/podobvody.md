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

## Knihovny prvků

Hotová zapojení najdete v oblasti **Knihovna součástek** pod nadpisem **Knihovny prvků**. Prvky jsou rozdělené podle kategorií (Kombinační obvody, Aritmetika, Sekvenční obvody…) a hledání je najde podle názvu i kategorie.

- **Vestavěná knihovna „Číslicová technika – výukové prvky“** obsahuje 36 zapojení běžných úloh: hradla z NAND a NOR, multiplexory, dekodér, komparátory, převodníky Grayova kódu, sčítačky a odčítačky, klopné obvody, čítače, registry, alarm a semafor.
- Prvek vložíte stejně jako součástku: klikněte na něj a pak do schématu. Objeví se u ukazatele myši a kliknutím ho položíte.
- Vložený prvek je **podobvod projektu** (kopie). Tabem ho otevřete, upravíte a uloží se s projektem. Prvky, které používá (např. úplné sčítačky ve 4bitové sčítačce), se vloží s ním.
- Pod vybraným prvkem uvidíte jeho popis, vývody, autora a licenci.
- **Osobní knihovny** jsou soubory `.lflib` ve složce `Dokumenty/LogicFlow/Knihovny`. Tlačítko **Otevřít složku** ji vytvoří a otevře, po zkopírování souborů klikněte na **Načíst znovu**. Jinou složku zvolíte v **Úpravy → Nastavení…** (tlačítko **Složky knihoven…**).
- **Sdílené složky** (síť, OneDrive, kroužek) přidáte v **Úpravy → Nastavení… → Knihovny**; jejich knihovny se ukážou s označením **Sdílená složka**.

> **Připravuje se:** export podobvodu do knihovny a školní knihovny.

> **Hotový příklad:** Soubor → Příklady → Úplná sčítačka ze dvou podobvodů.
