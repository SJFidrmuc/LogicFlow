//! Vestavěná nápověda (F1 nebo oblast typu „Nápověda“).
//!
//! [en] Built-in help (F1 or an area of type “Help”).

use crate::i18n::{Lang, T, tr};
use crate::theme::colors;
use egui::{RichText, Ui};

/// Stavební prvek kapitoly nápovědy.
///
/// [en] Building block of a help chapter.
enum Block {
    /// Podnadpis.
    ///
    /// [en] Subheading.
    H(&'static str),
    /// Odstavec.
    ///
    /// [en] Paragraph.
    P(&'static str),
    /// Odrážky.
    ///
    /// [en] Bullet points.
    List(&'static [&'static str]),
    /// Číslovaný postup.
    ///
    /// [en] Numbered steps.
    Steps(&'static [&'static str]),
    /// Tabulka klávesa > akce.
    ///
    /// [en] Table key > action.
    Keys(&'static [(&'static str, &'static str)]),
    /// Zvýrazněný tip.
    ///
    /// [en] Highlighted tip.
    Tip(&'static str),
}

/// Kapitola nápovědy.
///
/// [en] Help chapter.
struct Section {
    /// Název kapitoly.
    ///
    /// [en] Chapter title.
    title: &'static str,
    /// Obsah kapitoly.
    ///
    /// [en] Chapter content.
    blocks: &'static [Block],
}

use Block::*;

/// Kapitoly české nápovědy.
///
/// [en] Chapters of the Czech help.
const CS: &[Section] = &[
    Section {
        title: "Začínáme",
        blocks: &[
            P(
                "LogicFlow je simulátor logických obvodů pro výuku číslicové techniky. Obvod nakreslíte ze součástek a vodičů a hned vidíte, jak pracuje: vodiče mění barvu podle logické hodnoty a LED svítí.",
            ),
            H("První obvod za minutu"),
            Steps(&[
                "Soubor > Nový (Ctrl+N) otevře prázdné schéma.",
                "Najeďte myší do schématu a stiskněte Shift+A. Vyberte Přepínač a klikněte do schématu. Totéž ještě jednou – vzniknou vstupy A a B.",
                "Shift+A > AND [&] a umístěte hradlo vpravo od přepínačů. Pak Shift+A > LED a umístěte ji vpravo od hradla.",
                "Najeďte na výstup přepínače A (bod u pravé strany, zvýrazní se kroužkem), stiskněte levé tlačítko a táhněte ke vstupu hradla. Pusťte – vznikne vodič. Stejně propojte B a výstup hradla s LED.",
                "Stiskněte F5 (spustí simulaci) a klikejte na přepínače. LED se rozsvítí, jen když jsou oba vstupy 1. Esc vás vrátí k úpravám.",
            ]),
            Tip(
                "Hotové obvody najdete v nabídce Soubor > Příklady. Klopný obvod RS z hradel NOR je dobrá ukázka toho, jak si obvod pamatuje stav.",
            ),
        ],
    },
    Section {
        title: "Okno a oblasti",
        blocks: &[
            P(
                "Okno je stejně jako v Blenderu rozdělené na oblasti. Každá oblast má nahoře záhlaví a v něm volbu typu editoru: Schéma, Časový diagram, Vlastnosti, Hierarchie, Knihovna součástek, Pravdivostní tabulka, Editor paměti nebo Nápověda. Typ jde kdykoli přepnout.",
            ),
            H("Přidání a zavření oblasti"),
            List(&[
                "Vpravo v záhlaví oblasti je nabídka Oblast (nebo klikněte na záhlaví pravým tlačítkem).",
                "Rozdělit vedle sebe / Rozdělit pod sebe vytvoří novou oblast se stejným editorem; v ní pak přepněte typ.",
                "Zavřít oblast ji odstraní; poslední oblast zavřít nejde.",
                "Velikost oblastí změníte tažením hranice mezi nimi.",
                "Oblast přesunete tažením za prázdné místo v záhlaví; když ji pustíte doprostřed jiné oblasti, vzniknou záložky.",
            ]),
            H("Pracovní plochy"),
            P(
                "Záložky Návrh, Simulace, Analýza a Paměť v horní liště jsou pracovní plochy – každá má vlastní rozložení oblastí, ale všechny pracují se stejným obvodem. Přepnutím na plochu Simulace se simulace spustí a schéma se zamkne, odchodem z ní se zastaví. Přepínají se i klávesami Ctrl+PageUp a Ctrl+PageDown. Zobrazit > Obnovit rozložení oblastí vrátí výchozí stav.",
            ),
            Tip(
                "Každý editor schématu má vlastní pohled. Můžete mít v jedné oblasti celek a ve druhé přiblížený detail.",
            ),
        ],
    },
    Section {
        title: "Kreslení schématu",
        blocks: &[
            H("Přidání součástky"),
            List(&[
                "Shift+A nad schématem otevře nabídku Přidat. Nebo klikněte na součástku v Knihovně součástek a pak do schématu.",
                "Při umisťování klávesa R otáčí o 90°, Shift+klik umístí další kus, Esc nebo pravé tlačítko zruší.",
            ]),
            H("Vodiče"),
            List(&[
                "Vodič začíná tažením z vývodu součástky (vývod se pod myší zvýrazní kroužkem) nebo z libovolného bodu existujícího vodiče. S klávesou Ctrl jde začít kdekoli.",
                "Vodič se láme do pravého úhlu. Směr prvního úseku určuje první pohyb myši.",
                "Kde konec vodiče leží na jiném vodiči, vznikne spoj a zobrazí se tečka. Vodiče, které se jen kříží (bez tečky), propojené nejsou.",
                "Vývod součástky se připojí jen ke konci vodiče. Vodič, který přes vývod jen prochází, se s ním nespojí.",
                "Při kreslení vodiče oranžový kroužek ukazuje místo, kde by se vodič napojil i jinde než na začátku a konci. Pokud má opačné zalomení méně takových míst, použije se samo.",
                "Při přesunu součástky se připojené vodiče zalomí tak, aby se nedotkly jiného spoje. Když to nejde, vodič se od vývodu odpojí – volný konec je vidět a stačí ho dotáhnout.",
            ]),
            H("Úpravy"),
            Keys(&[
                ("klik", "vybrat součástku nebo vodič"),
                ("Shift+klik", "přidat do výběru / odebrat z výběru"),
                ("tažení v prázdném místě", "výběr obdélníkem"),
                ("tažení součástky, G", "přesun výběru (vodiče se přepojí)"),
                ("R", "otočit o 90°"),
                ("X, Delete", "smazat výběr"),
                ("Shift+D", "duplikovat výběr"),
                (
                    "Ctrl+C / X / V",
                    "kopírovat / vyjmout / vložit k ukazateli myši (i mezi listy a okny, s podobvody)",
                ),
                ("A / Alt+A", "vybrat vše / zrušit výběr"),
                ("Ctrl+Z / Ctrl+Shift+Z", "zpět / znovu"),
            ]),
            P(
                "V oblasti Vlastnosti jde u vybrané součástky změnit popisek, druh hradla, počet vstupů (2–8), hodnotu konstanty a otočení.",
            ),
            H("Kontextová nabídka (pravé tlačítko)"),
            List(&[
                "Kliknutí pravým tlačítkem (bez tažení) otevře nabídku podle místa: u součástky Přepnout, Frekvence, Počet vstupů, popisek, Otočit, Duplikovat, Přesunout, Smazat; u vodiče Vybrat celou síť, Smazat úsek, Smazat celou síť; v prázdném místě Přidat součástku sem, Vybrat vše, Celé schéma a ovládání simulace.",
                "Tažení pravým tlačítkem dál posouvá pohled; během rozdělané akce pravé tlačítko akci zruší.",
                "V režimu simulace jsou úpravy zašedlé. Pravé tlačítko funguje i v Hierarchii a v pravdivostní tabulce (Kopírovat jako text).",
            ]),
        ],
    },
    Section {
        title: "Součástky",
        blocks: &[
            Keys(&[
                (
                    "Přepínač",
                    "vstup 0 nebo 1; v simulaci se přepíná kliknutím, v režimu úprav dvojklikem",
                ),
                ("Tlačítko", "vstup, dává 1 jen po dobu stisku"),
                ("Konstanta 0 / 1", "pevná logická hodnota"),
                ("BUF  [1]", "opakovač (budič)"),
                ("NOT  [1] s trojúhelníkem", "invertor"),
                ("AND  [&]", "logický součin – 1, když jsou všechny vstupy 1"),
                (
                    "OR  [≥1]",
                    "logický součet – 1, když je aspoň jeden vstup 1",
                ),
                (
                    "NAND, NOR",
                    "negovaný součin / součet (trojúhelník na výstupu)",
                ),
                ("XOR  [=1]", "nonekvivalence – 1 při lichém počtu jedniček"),
                ("XNOR", "ekvivalence – negovaný XOR"),
                ("LED", "výstup, svítí při 1; při X nebo Z ukáže písmeno"),
            ]),
            P(
                "Značky hradel odpovídají normě ČSN EN 60617-12 (obdélníkové značky). Negace se značí indikátorem negace – malým pravoúhlým trojúhelníkem na vývodu.",
            ),
            H("Americké značky"),
            P(
                "V nabídce Zobrazit > Značky jde přepnout na americké tvarové značky podle ANSI/IEEE Std 91 (AND jako „D“, OR jako štít, negace kroužkem). Volba platí jen pro váš počítač – do souboru obvodu se neukládá, takže obvod otevřený jinde se zobrazí podle tamního nastavení. Při prvním spuštění se jazyk a značky nastaví podle systému: čeština pro české a slovenské prostředí, jinak angličtina, v americkém prostředí americké značky. Obojí jde kdykoli změnit v nabídce Zobrazit.",
            ),
            Tip("Paměti ROM a RAM a obvody řady 74xx přibudou v dalších verzích."),
        ],
    },
    Section {
        title: "Sekvenční obvody",
        blocks: &[
            P(
                "Sekvenční obvody si pamatují stav. Jejich výstup závisí nejen na vstupech, ale i na tom, co se dělo předtím. LogicFlow má hotové klopné obvody (Shift+A > Klopné obvody) a generátor hodin (Shift+A > Vstupy).",
            ),
            H("Klopné obvody"),
            Keys(&[
                (
                    "RS",
                    "S = 1 nastaví Q = 1, R = 1 vynuluje Q = 0, S = R = 0 drží stav, S = R = 1 je zakázaný stav (X)",
                ),
                (
                    "D (úroveň)",
                    "dokud je C = 1, výstup Q sleduje vstup D; při C = 0 drží",
                ),
                ("D", "při hraně hodin uloží hodnotu D"),
                (
                    "JK",
                    "při hraně: J = 1 nastaví, K = 1 vynuluje, J = K = 1 překlopí, J = K = 0 drží",
                ),
                ("T", "při hraně: T = 1 překlopí, T = 0 drží"),
            ]),
            P(
                "Každý klopný obvod má výstupy Q a Qn (negovaný). Vstup řízený hranou je ve značce označený trojúhelníkem uvnitř obrysu; indikátor negace (malý trojúhelník) na vývodu před ním znamená sestupnou hranu. V oblasti Vlastnosti jde zvolit druh obvodu, spouštění náběžnou nebo sestupnou hranou a počáteční stav (0 nebo 1).",
            ),
            H("Generátor hodin"),
            List(&[
                "Vytváří pravidelný obdélníkový signál. Frekvence se nastavuje ve Vlastnostech: rychlé volby, posuvník 0,1–100 Hz (nebo napsat číslo) nebo perioda v ms. Jde ji měnit i za běhu simulace, hodiny navážou bez skoku.",
                "Hodiny běží jen v režimu simulace (F5). Šipka vpravo = jeden takt, Shift+šipka vpravo = další hrana hodin, Ctrl+šipka vpravo = celá perioda (další náběžná hrana).",
                "Rychlost simulace ×0,1 až ×10 se nastavuje ve stavovém řádku nebo v nabídce Simulace > Rychlost simulace.",
                "V časovém diagramu jsou hodiny vždy nahoře, takže je vidět, při které hraně se výstupy mění.",
            ]),
            H("Restart simulace"),
            P(
                "Simulace > Restartovat simulaci (Ctrl+R) vrátí všechny klopné obvody do počátečního stavu, hodiny na 0 a vymaže časový diagram.",
            ),
            Tip(
                "Vyzkoušejte příklady Čítač 0–7 z klopných obvodů T a Posuvný registr 4 bity. U čítače jsou výstupy zapojené tak, že každý další stupeň řídí výstup Qn předchozího – to je asynchronní čítač nahoru.",
            ),
        ],
    },
    Section {
        title: "Funkční bloky a zobrazovače",
        blocks: &[
            P(
                "Hotové bloky najdete v Shift+A > Funkční bloky. Vstupy jsou vlevo, výstupy vpravo a bity jsou seřazené shora od nejnižšího (D0, Q0 …). Počet bitů nebo adresových vstupů se mění ve Vlastnostech.",
            ),
            Keys(&[
                (
                    "Multiplexor  MUX",
                    "adresa S vybere jeden ze vstupů D a pošle ho na výstup Y",
                ),
                (
                    "Dekodér  X/Y",
                    "výstup Y s číslem adresy A je 1, ostatní 0; při EN = 0 jsou všechny 0 (s datovým signálem na EN pracuje jako demultiplexor)",
                ),
                (
                    "Prioritní kodér  HPRI",
                    "na výstupech A je číslo nejvyššího vstupu I, který je 1; V = 1 znamená, že je aktivní aspoň jeden vstup",
                ),
                (
                    "Sčítačka  Σ",
                    "sečte čísla A a B a přenos CI; výsledek S a přenos CO (1bitová je úplná sčítačka)",
                ),
                ("Komparátor  COMP", "porovná čísla A a B: A<B, A=B, A>B"),
                (
                    "Převodník BCD/7SEG",
                    "číslo 0–15 na vstupech D převede na segmenty a–g (10–15 jako A–F)",
                ),
                (
                    "Čítač  CTR",
                    "při náběžné hraně C přičte (+) nebo odečte (−) jedničku, R nuluje okamžitě, EN = 0 čítání zastaví; CO = 1 při stavu 15 (dolů při 0)",
                ),
                (
                    "Registr  RG",
                    "při náběžné hraně C uloží vstupy D, R nuluje",
                ),
                (
                    "Posuvný registr  SRG",
                    "při náběžné hraně C posune obsah o bit dál (Q0 → Q1 …) a do Q0 zapíše SI",
                ),
            ]),
            Tip(
                "Vstupy R a EN můžete nechat nepřipojené: nepřipojený R znamená „nenulovat“ a nepřipojený EN „povoleno“. U ostatních vstupů dává nepřipojený vývod neznámou hodnotu X.",
            ),
            H("Zobrazovače"),
            List(&[
                "Sedmisegmentovka má vstupy a–g a desetinnou tečku dp; každý segment svítí při 1.",
                "Hex displej ukáže číslici 0–F podle 4bitového vstupu D0–D3. Při neznámém vstupu jsou segmenty tmavě šedé.",
                "Ve Vlastnostech jde zvolit, na které straně má zobrazovač vývody (vlevo, vpravo, nahoře, dole). Číslice zůstane vždy rovně – na rozdíl od otočení klávesou R.",
            ]),
            Tip(
                "Příklad Dekadický čítač se sedmisegmentovkou ukazuje čítač nulovaný hradlem AND při stavu 10 (1010) – takto vzniká čítač do libovolného čísla.",
            ),
        ],
    },
    Section {
        title: "Podobvody",
        blocks: &[
            P(
                "Podobvod je samostatný obvod, který vložíte do jiného jako jednu součástku – například poloviční sčítačku použijete dvakrát v úplné sčítačce. Vstupy podobvodu jsou jeho přepínače a tlačítka, výstupy jeho LED. Pořadí vývodů na značce určují jejich popisky (A, B, C …).",
            ),
            H("Postup"),
            Steps(&[
                "V oblasti Hierarchie klikněte na + Nový podobvod. Otevře se prázdný list a v záhlaví schématu uvidíte cestu Hlavní obvod / Podobvod 1.",
                "Nakreslete obvod se vstupy (přepínači) a výstupy (LED) a vyzkoušejte ho – i s pravdivostní tabulkou.",
                "V Hierarchii ho pojmenujte (pole Název).",
                "Tabulátorem se vrátíte do hlavního obvodu. Podobvod pak vložíte z Shift+A > Podobvody nebo z knihovny.",
            ]),
            H("Práce s podobvody"),
            List(&[
                "Tab nad vybranou značkou podobvodu ho otevře, další Tab vrátí zpět (jako skupiny uzlů v Blenderu).",
                "Úpravy podobvodu se projeví ve všech jeho instancích. Když přidáte vstup nebo výstup, značka dostane další vývod.",
                "Podobvod může obsahovat další podobvody, ale ne sám sebe.",
                "Použitý podobvod nejde smazat – nejdřív odstraňte jeho instance.",
                "Hodiny je potřeba přivést zvenku přes vstup podobvodu; generátor hodin uvnitř podobvodu stojí.",
            ]),
            Tip("Hotový příklad: Soubor > Příklady > Úplná sčítačka ze dvou podobvodů."),
        ],
    },
    Section {
        title: "Simulace",
        blocks: &[
            P(
                "Každé hradlo má zpoždění jeden takt, takže signál se obvodem šíří postupně – stejně jako ve skutečném obvodu.",
            ),
            H("Režim úprav a režim simulace"),
            List(&[
                "Režim úprav: schéma kreslíte a upravujete, logika se přepočítává hned, ale hodiny stojí. Přepínač se přepne dvojklikem, tlačítko druhým stiskem dvojkliku.",
                "Režim simulace spustí F5 (nebo mezerník) nebo přepnutí na plochu Simulace. Schéma je zamčené, fungují jen přepínače, tlačítka, posun, zoom, Tab a frekvence hodin ve Vlastnostech. Schéma má jantarový rámeček se štítkem SIMULACE.",
                "Esc nebo Shift+F5 vrátí do režimu úprav. Stav obvodu zůstane, vynuluje ho jen Restartovat simulaci (Ctrl+R).",
                "Úpravy za běhu povolí Simulace > Povolit úpravy za běhu simulace (výchozí vypnuto).",
            ]),
            H("Barvy vodičů"),
            Keys(&[
                ("světle zelená", "logická 1"),
                ("tmavě zelená", "logická 0"),
                (
                    "červená (X)",
                    "neurčitá hodnota – nepřipojený vstup, kolize dvou výstupů, neinicializovaná paměť",
                ),
                ("modrá (Z)", "nepřipojený vodič (vysoká impedance)"),
            ]),
            H("Krokování"),
            List(&[
                "Mezerník v režimu úprav simulaci spustí, v simulaci ji pozastaví nebo nechá pokračovat.",
                "Šipka vpravo (nebo Simulace > Krok) provede jeden takt; z režimu úprav přejde do pozastavené simulace. Je vidět, jak změna prochází hradlo po hradle.",
                "Pokud se obvod nikdy neustálí (např. invertor zapojený sám do sebe), zobrazí se upozornění „Obvod kmitá“.",
                "Simulace > Restartovat simulaci (Ctrl+R) začne znovu od počátečního stavu.",
            ]),
            Tip(
                "Obvody s pamětí, například klopný obvod RS z hradel NOR, začínají ve stavu X. Teprve nastavení (S) nebo nulování (R) určí jejich stav.",
            ),
        ],
    },
    Section {
        title: "Analýza",
        blocks: &[
            H("Časový diagram"),
            P(
                "Zaznamenává průběh všech přepínačů, tlačítek a LED za posledních 20 sekund. Signály jsou seřazené podle popisku. Při pozastavené simulaci posune každý krok diagram o čtvrt sekundy.",
            ),
            H("Pravdivostní tabulka"),
            P(
                "Spočítá se simulací všech kombinací vstupů (nejvýše 8 vstupů, tj. 256 řádků). Vstupy jsou přepínače a tlačítka, výstupy LED. Zvýrazněný řádek odpovídá aktuálnímu nastavení přepínačů. U obvodů s pamětí tabulka nemusí dávat smysl.",
            ),
        ],
    },
    Section {
        title: "Soubory",
        blocks: &[List(&[
            "Ctrl+S uloží obvod, Ctrl+Shift+S uloží pod jiným názvem, Ctrl+O otevře, Ctrl+N vytvoří nový.",
            "Obvody se ukládají do souborů .logicflow i se všemi podobvody. Je to čitelný text (JSON), takže se dá odevzdat, poslat e-mailem nebo uložit do Gitu.",
            "Okno pro výběr souboru má tlačítka pro domovskou složku a Dokumenty; do řádku s cestou jde napsat libovolnou složku. Tlačítko + Nová složka založí podsložku (třeba pro třídu nebo úlohu) a rovnou do ní přejde.",
            "Před zavřením nebo otevřením jiného obvodu se aplikace zeptá na neuložené změny.",
            "Soubor jde otevřít i z příkazové řádky: logicflow obvod.logicflow",
        ])],
    },
    Section {
        title: "Klávesové zkratky",
        blocks: &[Keys(&[
            ("F1", "nápověda"),
            ("Shift+A", "přidat součástku"),
            ("G", "přesunout výběr"),
            ("R", "otočit"),
            ("X, Delete", "smazat"),
            ("Shift+D", "duplikovat"),
            ("Ctrl+C / X / V", "kopírovat / vyjmout / vložit"),
            ("A / Alt+A", "vybrat vše / nic"),
            (
                "Esc",
                "zrušit nástroj nebo výběr; v simulaci zpět k úpravám",
            ),
            ("Home", "zobrazit celé schéma"),
            ("Tab", "otevřít vybraný podobvod / zpět o úroveň výš"),
            ("kolečko myši", "zoom"),
            ("prostřední / pravé tlačítko + tažení", "posun pohledu"),
            ("F5", "spustit simulaci"),
            ("Shift+F5", "zastavit simulaci, zpět k úpravám"),
            ("Mezerník", "spustit / pozastavit / pokračovat"),
            ("šipka vpravo", "jeden takt (při pozastavení)"),
            ("Shift+šipka vpravo", "krok na další hranu hodin"),
            ("Ctrl+šipka vpravo", "krok o periodu hodin"),
            ("Ctrl+R", "restartovat simulaci"),
            ("Ctrl+Z / Ctrl+Shift+Z, Ctrl+Y", "zpět / znovu"),
            ("Ctrl+N / O / S", "nový / otevřít / uložit"),
            ("Ctrl+Shift+S", "uložit jako"),
            (
                "Ctrl+PageUp / PageDown",
                "předchozí / další pracovní plocha",
            ),
            ("Ctrl+Q", "ukončit"),
        ])],
    },
    Section {
        title: "Windows hlásí nebezpečnou aplikaci",
        blocks: &[
            P(
                "Windows SmartScreen upozorní na každý program stažený z internetu, který zatím nemá digitální podpis a „pověst“. LogicFlow je otevřený program, jehož zdrojový kód je veřejně na GitHubu, a hotové soubory sestavuje automaticky GitHub.",
            ),
            H("Jak program spustit"),
            Steps(&[
                "V okně „Systém Windows ochránil váš počítač“ klikněte na Další informace.",
                "Zobrazí se tlačítko Přesto spustit – klikněte na něj.",
            ]),
            H("Aby se upozornění neopakovalo"),
            Steps(&[
                "Klikněte na stažený soubor logicflow.exe (nebo ZIP před rozbalením) pravým tlačítkem a zvolte Vlastnosti.",
                "Dole na kartě Obecné zaškrtněte Odblokovat a potvrďte OK.",
            ]),
            Tip(
                "Ve škole může program povolit správce sítě pro všechny počítače najednou. Podepsaná verze programu je v plánu.",
            ),
        ],
    },
];

/// Kapitoly anglické nápovědy (stejná struktura jako česká – hlídá test).
///
/// [en] Chapters of the English help (same structure as the Czech one – checked by a test).
const EN: &[Section] = &[
    Section {
        title: "Getting started",
        blocks: &[
            P(
                "LogicFlow is a logic circuit simulator for teaching digital electronics. Draw a circuit from components and wires and watch it work right away: wires change colour with their logic value and LEDs light up.",
            ),
            H("Your first circuit in a minute"),
            Steps(&[
                "File > New (Ctrl+N) opens an empty schematic.",
                "Move the mouse over the schematic and press Shift+A. Choose Switch and click in the schematic. Repeat – you get inputs A and B.",
                "Shift+A > AND [&] and place the gate to the right of the switches. Then Shift+A > LED and place it right of the gate.",
                "Hover the output of switch A (the point on its right side gets a ring), press the left button and drag to a gate input. Release – a wire is created. Connect B and the gate output to the LED the same way.",
                "Press F5 (starts the simulation) and click the switches. The LED lights only when both inputs are 1. Esc takes you back to editing.",
            ]),
            Tip(
                "Ready-made circuits are in File > Examples. The SR latch built from NOR gates shows nicely how a circuit remembers its state.",
            ),
        ],
    },
    Section {
        title: "Window and areas",
        blocks: &[
            P(
                "Like in Blender, the window is divided into areas. Every area has a header with an editor type selector: Schematic, Timing diagram, Properties, Hierarchy, Component library, Truth table, Memory editor or Help. You can switch the type at any time.",
            ),
            H("Adding and closing areas"),
            List(&[
                "On the right of an area header there is the Area menu (or right-click the header).",
                "Split side by side / Split below creates a new area with the same editor; switch its type afterwards.",
                "Close area removes it; the last area cannot be closed.",
                "Drag the border between areas to resize them.",
                "Drag an area by the empty part of its header; dropping it onto the middle of another area creates tabs.",
            ]),
            H("Workspaces"),
            P(
                "The Design, Simulation, Analysis and Memory tabs in the top bar are workspaces – each has its own area layout but all work with the same circuit. Switching to the Simulation workspace starts the simulation and locks the schematic, leaving it stops the simulation. Ctrl+PageUp and Ctrl+PageDown switch them too. View > Reset area layout restores the default.",
            ),
            Tip(
                "Every schematic editor has its own view: one area can show the whole circuit while another zooms into a detail.",
            ),
        ],
    },
    Section {
        title: "Drawing schematics",
        blocks: &[
            H("Adding components"),
            List(&[
                "Shift+A over the schematic opens the Add menu. Or click a component in the Component library and then in the schematic.",
                "While placing, R rotates by 90°, Shift+click places another one, Esc or the right button cancels.",
            ]),
            H("Wires"),
            List(&[
                "A wire starts by dragging from a component pin (highlighted with a ring) or from any point of an existing wire. With Ctrl held you can start anywhere.",
                "Wires bend at right angles. The first mouse movement decides the direction of the first segment.",
                "Where a wire end lies on another wire, a connection with a dot is made. Wires that merely cross (no dot) are not connected.",
                "A component pin connects only to a wire end; a wire merely passing over a pin is not connected to it.",
                "While drawing a wire, an orange ring marks where it would also connect besides its start and end. If the opposite bend has fewer such places, it is used automatically.",
                "When a component is moved, attached wires are bent so they touch no other net. If that is impossible, the wire is detached from the pin – the loose end is visible and easy to reconnect.",
            ]),
            H("Editing"),
            Keys(&[
                ("click", "select a component or wire"),
                ("Shift+click", "add to / remove from selection"),
                ("drag on empty space", "box select"),
                ("drag a component, G", "move the selection (wires follow)"),
                ("R", "rotate by 90°"),
                ("X, Delete", "delete selection"),
                ("Shift+D", "duplicate selection"),
                (
                    "Ctrl+C / X / V",
                    "copy / cut / paste at the mouse pointer (also between sheets and windows, with subcircuits)",
                ),
                ("A / Alt+A", "select all / deselect"),
                ("Ctrl+Z / Ctrl+Shift+Z", "undo / redo"),
            ]),
            P(
                "The Properties area lets you change the label, gate type, number of inputs (2–8), constant value and rotation of the selected component.",
            ),
            H("Context menu (right button)"),
            List(&[
                "A right-click (without dragging) opens a menu depending on the place: on a component Toggle, Frequency, Inputs, label, Rotate, Duplicate, Move, Delete; on a wire Select the whole net, Delete segment, Delete the whole net; on an empty place Add component here, Select all, Fit view and simulation controls.",
                "Dragging with the right button still pans; during an action in progress the right button cancels it.",
                "In simulation mode edits are greyed out. The right button also works in the Hierarchy and in the truth table (Copy as text).",
            ]),
        ],
    },
    Section {
        title: "Components",
        blocks: &[
            Keys(&[
                (
                    "Switch",
                    "input 0 or 1; toggled by a click in simulation, by a double-click in edit mode",
                ),
                ("Push button", "input, 1 only while pressed"),
                ("Constant 0 / 1", "fixed logic value"),
                ("BUF  [1]", "buffer"),
                ("NOT  [1] with triangle", "inverter"),
                ("AND  [&]", "1 when all inputs are 1"),
                ("OR  [≥1]", "1 when at least one input is 1"),
                ("NAND, NOR", "negated AND / OR (triangle on the output)"),
                ("XOR  [=1]", "1 for an odd number of ones"),
                ("XNOR", "negated XOR"),
                ("LED", "output, lit at 1; shows a letter for X or Z"),
            ]),
            P(
                "Gate symbols follow IEC 60617-12 (rectangular symbols). Negation is shown by the polarity indicator – a small right triangle on the pin.",
            ),
            H("American symbols"),
            P(
                "View > Symbols switches to American distinctive-shape symbols (ANSI/IEEE Std 91: AND as a “D”, OR as a shield, negation as a bubble). The choice is stored on your computer only, not in the circuit file. On first start, language and symbols follow the system: Czech for Czech and Slovak systems, English otherwise, American symbols in a US environment. Both can be changed any time in the View menu.",
            ),
            Tip("ROM and RAM memories and 74xx chips arrive in later versions."),
        ],
    },
    Section {
        title: "Sequential circuits",
        blocks: &[
            P(
                "Sequential circuits remember their state: the output depends not only on the inputs but also on what happened before. LogicFlow has ready-made flip-flops (Shift+A > Flip-flops) and a clock generator (Shift+A > Inputs).",
            ),
            H("Flip-flops"),
            Keys(&[
                (
                    "RS",
                    "S = 1 sets Q = 1, R = 1 resets Q = 0, S = R = 0 holds, S = R = 1 is forbidden (X)",
                ),
                ("D (level)", "while C = 1, Q follows D; with C = 0 it holds"),
                ("D", "stores D on the clock edge"),
                (
                    "JK",
                    "on the edge: J = 1 sets, K = 1 resets, J = K = 1 toggles, J = K = 0 holds",
                ),
                ("T", "on the edge: T = 1 toggles, T = 0 holds"),
            ]),
            P(
                "Every flip-flop has outputs Q and Qn (negated). An edge-triggered input is marked with a triangle inside the outline; a polarity indicator (small triangle) on its pin means the falling edge. In Properties you can choose the type, rising or falling edge and the initial state (0 or 1).",
            ),
            H("Clock generator"),
            List(&[
                "Produces a regular square wave. Set the frequency in Properties: quick choices, a slider 0.1–100 Hz (or type a number) or the period in ms. It can be changed while simulating; the clock continues without a jump.",
                "The clock runs only in simulation mode (F5). Right arrow = one tick, Shift+Right arrow = next clock edge, Ctrl+Right arrow = one whole period (next rising edge).",
                "Simulation speed ×0.1 to ×10 is set in the status bar or in Simulation > Simulation speed.",
                "Clocks are always at the top of the timing diagram, so you can see on which edge the outputs change.",
            ]),
            H("Restarting the simulation"),
            P(
                "Simulation > Restart simulation (Ctrl+R) returns all flip-flops to their initial state, sets clocks to 0 and clears the timing diagram.",
            ),
            Tip(
                "Try the examples 0–7 counter from T flip-flops and 4-bit shift register. In the counter each stage is clocked by Qn of the previous one – an asynchronous up counter.",
            ),
        ],
    },
    Section {
        title: "Function blocks and displays",
        blocks: &[
            P(
                "Ready-made blocks are under Shift+A > Function blocks. Inputs are on the left, outputs on the right, bits are ordered from the least significant at the top (D0, Q0 …). The number of bits or select inputs is set in Properties.",
            ),
            Keys(&[
                (
                    "Multiplexer  MUX",
                    "select S routes one of the D inputs to Y",
                ),
                (
                    "Decoder  X/Y",
                    "output Y with the number on A is 1, others 0; EN = 0 clears all (with data on EN it works as a demultiplexer)",
                ),
                (
                    "Priority encoder  HPRI",
                    "A shows the number of the highest active input I; V = 1 when any input is active",
                ),
                (
                    "Adder  Σ",
                    "adds A, B and carry CI; sum S and carry CO (the 1-bit one is a full adder)",
                ),
                ("Comparator  COMP", "compares A and B: A<B, A=B, A>B"),
                (
                    "BCD/7SEG decoder",
                    "turns 0–15 on D into segments a–g (10–15 as A–F)",
                ),
                (
                    "Counter  CTR",
                    "on a rising edge of C counts up (+) or down (−), R clears at once, EN = 0 stops counting; CO = 1 at 15 (0 when counting down)",
                ),
                ("Register  RG", "stores D on a rising edge of C, R clears"),
                (
                    "Shift register  SRG",
                    "on a rising edge of C shifts by one bit (Q0 → Q1 …) and loads SI into Q0",
                ),
            ]),
            Tip(
                "R and EN may stay unconnected: unconnected R means “no reset”, unconnected EN means “enabled”. Other unconnected inputs read as unknown X.",
            ),
            H("Displays"),
            List(&[
                "The seven-segment display has inputs a–g and the decimal point dp; a segment is lit at 1.",
                "The hex display shows 0–F from the 4-bit input D0–D3; unknown input shows grey segments.",
                "In Properties you can choose the side of the pins (left, right, top, bottom); the digit stays upright, unlike rotating with R.",
            ]),
            Tip(
                "The example Decade counter with a seven-segment display resets a counter with an AND gate at 10 (1010) – this is how a counter to any number is made.",
            ),
        ],
    },
    Section {
        title: "Subcircuits",
        blocks: &[
            P(
                "A subcircuit is a circuit you insert into another one as a single component – e.g. a half adder used twice in a full adder. Its inputs are its switches and buttons, its outputs its LEDs; their labels set the pin order on the symbol.",
            ),
            H("How to"),
            Steps(&[
                "In the Hierarchy area click + New subcircuit. An empty sheet opens; the schematic header shows Main circuit / Subcircuit 1.",
                "Draw the circuit with switches and LEDs and test it, truth table included.",
                "Name it in the Hierarchy (Name field).",
                "Press Tab to return to the main circuit and insert the subcircuit from Shift+A > Subcircuits or the library.",
            ]),
            H("Working with subcircuits"),
            List(&[
                "Tab over a selected subcircuit opens it, another Tab goes back (like node groups in Blender).",
                "Changes apply to every instance; a new input or output adds a pin to the symbol.",
                "Subcircuits may contain other subcircuits, but not themselves.",
                "A subcircuit in use cannot be deleted – remove its instances first.",
                "Feed clocks in through a subcircuit input; a clock generator inside a subcircuit does not run.",
            ]),
            Tip("Ready-made example: File > Examples > Full adder from two subcircuits."),
        ],
    },
    Section {
        title: "Simulation",
        blocks: &[
            P(
                "Each gate has a delay of one tick, so a signal travels through the circuit step by step – just like in a real circuit.",
            ),
            H("Edit mode and simulation mode"),
            List(&[
                "Edit mode: you draw and edit the schematic, logic is recomputed at once, but clocks stand still. A switch is toggled by a double-click, a button by the second press of a double-click.",
                "Simulation mode is started with F5 (or Space) or by switching to the Simulation workspace. The schematic is locked; only switches, buttons, panning, zooming, Tab and the clock frequency in Properties work. The schematic has an amber frame with a SIMULATION badge.",
                "Esc or Shift+F5 returns to edit mode. The circuit state stays; only Restart simulation (Ctrl+R) clears it.",
                "Simulation > Allow editing while simulating permits edits while running (off by default).",
            ]),
            H("Wire colours"),
            Keys(&[
                ("light green", "logic 1"),
                ("dark green", "logic 0"),
                (
                    "red (X)",
                    "undefined – unconnected input, two outputs colliding, uninitialised memory",
                ),
                ("blue (Z)", "unconnected wire (high impedance)"),
            ]),
            H("Stepping"),
            List(&[
                "Space starts the simulation in edit mode; in simulation it pauses or resumes it.",
                "The Right arrow (or Simulation > Step) performs one tick; from edit mode it goes to a paused simulation. You can watch a change pass gate by gate.",
                "If the circuit never settles (e.g. an inverter feeding itself), an “oscillating” warning appears.",
                "Simulation > Restart simulation (Ctrl+R) starts again from the initial state.",
            ]),
            Tip(
                "Circuits with memory, such as an SR latch made of NOR gates, start in state X. Only set (S) or reset (R) defines their state.",
            ),
        ],
    },
    Section {
        title: "Analysis",
        blocks: &[
            H("Timing diagram"),
            P(
                "Records all switches, buttons and LEDs over the last 20 seconds, sorted by label. While paused, every step moves the diagram by a quarter of a second.",
            ),
            H("Truth table"),
            P(
                "Computed by simulating every input combination (at most 8 inputs, i.e. 256 rows). Inputs are switches and buttons, outputs are LEDs. The highlighted row matches the current switch settings. For circuits with memory the table may not make sense.",
            ),
        ],
    },
    Section {
        title: "Files",
        blocks: &[List(&[
            "Ctrl+S saves, Ctrl+Shift+S saves under a new name, Ctrl+O opens, Ctrl+N creates a new circuit.",
            "Circuits are stored in .logicflow files. They are readable text (JSON), easy to hand in, e-mail or keep in Git.",
            "The file browser has buttons for the home folder and Documents; you can type any folder into the path line. The + New folder button creates a subfolder and opens it.",
            "Before closing or opening another circuit the app asks about unsaved changes.",
            "A file can also be opened from the command line: logicflow circuit.logicflow",
        ])],
    },
    Section {
        title: "Keyboard shortcuts",
        blocks: &[Keys(&[
            ("F1", "help"),
            ("Shift+A", "add component"),
            ("G", "move selection"),
            ("R", "rotate"),
            ("X, Delete", "delete"),
            ("Shift+D", "duplicate"),
            ("Ctrl+C / X / V", "copy / cut / paste"),
            ("A / Alt+A", "select all / none"),
            (
                "Esc",
                "cancel tool or selection; in simulation back to editing",
            ),
            ("Home", "fit the whole schematic"),
            ("Tab", "open selected subcircuit / go back one level"),
            ("mouse wheel", "zoom"),
            ("middle / right button + drag", "pan"),
            ("F5", "run the simulation"),
            ("Shift+F5", "stop the simulation, back to editing"),
            ("Space", "run / pause / resume"),
            ("Right arrow", "one tick (while paused)"),
            ("Shift+Right arrow", "step to the next clock edge"),
            ("Ctrl+Right arrow", "step one clock period"),
            ("Ctrl+R", "restart the simulation"),
            ("Ctrl+Z / Ctrl+Shift+Z, Ctrl+Y", "undo / redo"),
            ("Ctrl+N / O / S", "new / open / save"),
            ("Ctrl+Shift+S", "save as"),
            ("Ctrl+PageUp / PageDown", "previous / next workspace"),
            ("Ctrl+Q", "quit"),
        ])],
    },
    Section {
        title: "Windows says the app is unsafe",
        blocks: &[
            P(
                "Windows SmartScreen warns about every downloaded program that does not yet have a digital signature and reputation. LogicFlow is open source, its code is public on GitHub and the binaries are built automatically by GitHub.",
            ),
            H("How to run it"),
            Steps(&[
                "In the “Windows protected your PC” window click More info.",
                "A Run anyway button appears – click it.",
            ]),
            H("To stop the warning from coming back"),
            Steps(&[
                "Right-click the downloaded logicflow.exe (or the ZIP before extracting) and choose Properties.",
                "At the bottom of the General tab tick Unblock and confirm with OK.",
            ]),
            Tip(
                "At school, the network administrator can allow the program on all computers at once. A signed version is planned.",
            ),
        ],
    },
];

/// Kapitoly v daném jazyce.
///
/// [en] Chapters in the given language.
fn sections(lang: Lang) -> &'static [Section] {
    match lang {
        Lang::Cs => CS,
        Lang::En => EN,
    }
}

/// Vykreslí jednu kapitolu.
///
/// [en] Draws one chapter.
fn render(ui: &mut Ui, section: &Section) {
    ui.heading(section.title);
    ui.add_space(4.0);
    for block in section.blocks {
        match block {
            H(t) => {
                ui.add_space(8.0);
                ui.label(RichText::new(*t).strong().size(15.0));
            }
            P(t) => {
                ui.label(*t);
                ui.add_space(4.0);
            }
            List(items) => {
                for t in *items {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new("•").color(colors::ACCENT));
                        ui.label(*t);
                    });
                }
                ui.add_space(4.0);
            }
            Steps(items) => {
                for (i, t) in items.iter().enumerate() {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(
                            RichText::new(format!("{}.", i + 1))
                                .strong()
                                .color(colors::ACCENT),
                        );
                        ui.label(*t);
                    });
                }
                ui.add_space(4.0);
            }
            Keys(rows) => {
                egui::Grid::new(("help_keys", section.title, rows.len()))
                    .striped(true)
                    .num_columns(2)
                    .spacing([16.0, 4.0])
                    .show(ui, |ui| {
                        for (k, d) in *rows {
                            ui.label(RichText::new(*k).monospace().strong());
                            ui.label(*d);
                            ui.end_row();
                        }
                    });
                ui.add_space(4.0);
            }
            Tip(t) => {
                ui.add_space(4.0);
                egui::Frame::new()
                    .fill(colors::ACCENT.gamma_multiply(0.18))
                    .stroke(egui::Stroke::new(1.0, colors::ACCENT))
                    .corner_radius(4)
                    .inner_margin(8)
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui.label(*t);
                    });
            }
        }
    }
}

/// Okno nápovědy (F1).
///
/// [en] Help window (F1).
pub struct HelpWindow {
    /// Je okno otevřené?
    ///
    /// [en] Is the window open?
    pub open: bool,
    /// Pořadí zobrazené kapitoly.
    ///
    /// [en] Index of the chapter shown.
    pub section: usize,
}

impl HelpWindow {
    /// Zavřené okno na první kapitole.
    ///
    /// [en] Closed window at the first chapter.
    pub fn new() -> Self {
        Self {
            open: false,
            section: 0,
        }
    }

    /// Otevře nápovědu na kapitole podle jejího pořadí.
    ///
    /// [en] Opens the help at a chapter by its index.
    pub fn show_section(&mut self, section: usize) {
        self.open = true;
        self.section = section;
    }

    /// Vykreslí okno: seznam kapitol vlevo, obsah vpravo.
    ///
    /// [en] Draws the window: list of chapters on the left, content on the right.
    pub fn ui(&mut self, ctx: &egui::Context, lang: Lang) {
        if !self.open {
            return;
        }
        let secs = sections(lang);
        let mut open = self.open;
        egui::Window::new(tr(lang, T::Manual))
            .open(&mut open)
            .default_size([780.0, 540.0])
            .min_size([520.0, 300.0])
            .resizable(true)
            .collapsible(false)
            .show(ctx, |ui| {
                egui::Panel::left("help_nav")
                    .resizable(false)
                    .exact_size(200.0)
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .id_salt("help_nav_scroll")
                            .show(ui, |ui| {
                                for (i, s) in secs.iter().enumerate() {
                                    if ui
                                        .add(
                                            egui::Button::selectable(self.section == i, s.title)
                                                .wrap(),
                                        )
                                        .clicked()
                                    {
                                        self.section = i;
                                    }
                                }
                            });
                    });
                egui::CentralPanel::default_margins().show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt(("help_body", self.section))
                        .show(ui, |ui| {
                            if let Some(s) = secs.get(self.section) {
                                render(ui, s);
                            }
                        });
                });
            });
        self.open = open;
    }
}

/// Nápověda zobrazená v oblasti (typ editoru „Nápověda“).
///
/// [en] Help shown in an area (editor type “Help”).
pub fn area(ui: &mut Ui, section: &mut usize, lang: Lang) {
    let secs = sections(lang);
    egui::Frame::new().inner_margin(8).show(ui, |ui| {
        let current = secs.get(*section).map_or("", |s| s.title);
        egui::ComboBox::from_id_salt("help_area_section")
            .selected_text(current)
            .width(ui.available_width().min(320.0))
            .show_ui(ui, |ui| {
                for (i, s) in secs.iter().enumerate() {
                    ui.selectable_value(section, i, s.title);
                }
            });
        ui.add_space(6.0);
        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(ui, |ui| {
                if let Some(s) = secs.get(*section) {
                    render(ui, s);
                }
            });
    });
}

/// Pořadí kapitoly s klávesovými zkratkami a s upozorněním Windows (pro nabídku Nápověda).
///
/// [en] Index of the chapter with keyboard shortcuts and with the Windows warning (for the Help menu).
pub const SECTION_SHORTCUTS: usize = 10;
/// Pořadí kapitoly s upozorněním Windows (SmartScreen).
///
/// [en] Index of the chapter with the Windows (SmartScreen) warning.
pub const SECTION_WINDOWS: usize = 11;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn languages_have_same_structure() {
        assert_eq!(CS.len(), EN.len());
        for (c, e) in CS.iter().zip(EN) {
            assert_eq!(c.blocks.len(), e.blocks.len(), "kapitola {}", c.title);
        }
        assert_eq!(CS[SECTION_SHORTCUTS].title, "Klávesové zkratky");
        assert!(CS[SECTION_WINDOWS].title.starts_with("Windows"));
    }
}
