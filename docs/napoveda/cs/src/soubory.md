# Soubory, příklady a nastavení

## Ukládání a otevírání

- **Ctrl+S** uloží obvod, **Ctrl+Shift+S** uloží pod jiným názvem, **Ctrl+O** otevře, **Ctrl+N** vytvoří nový.
- Obvody se ukládají do souborů **.logicflow** i se všemi podobvody. Je to čitelný text (JSON), takže se dá odevzdat, poslat e-mailem nebo uložit do Gitu. Stejný soubor otevřete ve Windows, na Macu, v Linuxu i na Raspberry Pi.
- Okno pro výběr souboru má tlačítka pro domovskou složku a Dokumenty, do řádku s cestou jde napsat libovolnou složku. Tlačítko **+ Nová složka** založí podsložku (třeba pro třídu nebo úlohu) a rovnou do ní přejde.
- Před zavřením programu nebo otevřením jiného obvodu se program zeptá na neuložené změny.
- Soubor jde otevřít i z příkazové řádky: `logicflow obvod.logicflow`

## Příklady

V nabídce **Soubor → Příklady** je 9 hotových obvodů. Příklad se otevře jako nový obvod, takže ho můžete libovolně upravit a uložit pod vlastním názvem.

![Nabídka Soubor → Příklady](obrazky/priklady.png)

| Příklad | Co ukazuje |
| --- | --- |
| Hradla AND a OR | základní hradla a pravdivostní tabulku |
| Poloviční sčítačka | kombinační obvod z hradel XOR a AND |
| Klopný obvod RS z hradel NOR | paměť složenou z hradel a počáteční stav X |
| Klopný obvod D s hodinami | uložení hodnoty při hraně hodin |
| Čítač 0–7 z klopných obvodů T | asynchronní čítač a časový diagram |
| Posuvný registr 4 bity | posun dat řetězcem klopných obvodů D |
| Multiplexor 4:1 | funkční blok MUX |
| Dekadický čítač se sedmisegmentovkou | čítač, převodník BCD/7SEG a nulování při stavu 10 |
| Úplná sčítačka ze dvou podobvodů | podobvody a jejich vnoření |

## Jazyk a značky

- **Jazyk** přepnete tlačítky CS / EN vpravo nahoře nebo v nabídce **Zobrazit → Jazyk**. Změna platí hned, bez restartu.
- **Styl značek** (evropské podle ČSN EN 60617-12, nebo americké ANSI/IEEE) zvolíte v **Zobrazit → Značky**.
- Při prvním spuštění se obojí nastaví podle systému: čeština v českém a slovenském prostředí, jinak angličtina; americké značky v americkém prostředí. Vaši ruční volbu si program pamatuje v souboru s nastavením.

## Nastavení

Okno **Úpravy → Nastavení…** sdružuje osobní nastavení. Program si ho pamatuje v souboru s nastavením.

- **Obecné:** jazyk, styl značek a povolení úprav za běhu simulace (stejné volby jako v nabídkách).
- **Autor a licence:** vaše jméno a výchozí licence (CC BY 4.0). Použijí se u prvků exportovaných do knihovny a v informacích o obvodu.
- **Knihovny:** složka osobních knihoven (výchozí `Dokumenty/LogicFlow/Knihovny`) a **sdílené složky** – složky na síti, na OneDrivu nebo kroužku, jejichž knihovny uvidíte v Knihovně součástek. U sdílené složky volba **jen pro čtení** znamená, že do ní program nic nezapíše. Nedostupná složka (odpojená síť) se v knihovně ukáže jako upozornění.
- **Soubor s nastavením** je čitelný text `logicflow.toml`: ve Windows `%APPDATA%\LogicFlow` (s cestovním profilem se přenáší na další počítače), v macOS `~/Library/Application Support/LogicFlow`, v Linuxu a na Raspberry Pi `~/.config/logicflow`. Tlačítko **Otevřít složku s nastavením** ho ukáže ve správci souborů.
- Soubor jde upravit i ručně. Když ho program nedokáže přečíst, odloží ho jako `logicflow.toml.bak` a použije výchozí nastavení.
- **Obnovit výchozí nastavení…** (s potvrzením) vrátí jazyk a značky podle systému a smaže jméno autora, licenci i složky knihoven.

## Nabídka Nápověda

- **Nápověda** (F1) otevře tuto nápovědu v programu.
- **Webová stránka projektu** otevře www.logicflow-sim.cz.
- **O aplikaci** ukazuje verzi programu, licenci a odkazy na web a zdrojový kód. Verzi uvádějte při hlášení chyby.
