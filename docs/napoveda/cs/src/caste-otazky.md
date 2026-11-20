# Časté otázky

## Windows hlásí nebezpečnou aplikaci. Je program bezpečný?

Windows SmartScreen upozorní na každý program stažený z internetu, který zatím nemá digitální podpis a „pověst“. LogicFlow je otevřený program se zdrojovým kódem na GitHubu a hotové soubory sestavuje automaticky GitHub.

1. V okně *Systém Windows ochránil váš počítač* klikněte na **Další informace** a pak na **Přesto spustit**.
2. Aby se upozornění neopakovalo, klikněte na stažený ZIP před rozbalením pravým tlačítkem → **Vlastnosti**, zaškrtněte **Odblokovat** a potvrďte OK.

Ve škole může program povolit správce sítě pro všechny počítače najednou. Podepsaná verze programu je v plánu.

## Na Macu se program neotevře.

Program zatím není podepsaný pro macOS. Při prvním spuštění klikněte na `logicflow` pravým tlačítkem a zvolte **Otevřít**.

## Na Raspberry Pi program nejde spustit.

Potřebujete Raspberry Pi 4 nebo 5 a 64bitový Raspberry Pi OS. Na starších modelech a 32bitovém systému program neběží.

## Proč je vodič červený a LED ukazuje X?

Hodnota je neurčitá. Nejčastější příčinou je nepřipojený vstup hradla nebo dva výstupy spojené na jeden vodič. Obvody s pamětí začínají ve stavu X, dokud je nenastavíte nebo nevynulujete.

## Nakreslil jsem vodič, ale signál jím neprochází.

Zkontrolujte, zda je v místě spoje tečka. Vodiče, které se jen kříží, propojené nejsou. Vývod součástky se připojí jen ke konci vodiče, ne k vodiči, který přes něj jen vede.

## Pravdivostní tabulka je prázdná.

Tabulka potřebuje aspoň jeden vstup (přepínač nebo tlačítko) a jednu LED. Počítá se nejvýše pro 8 vstupů.

## Program hlásí „Obvod kmitá“.

Obvod nemá ustálený stav, například invertor zapojený sám do sebe nebo lichý počet invertorů v kruhu. Zkontrolujte zpětnou vazbu.

## Generátor hodin v podobvodu stojí.

To je záměr. Hodiny přiveďte zvenku přes vstup podobvodu.

## Jak poznám, jakou verzi mám?

Nápověda → O aplikaci, nebo vpravo dole ve stavovém řádku. Nejnovější verze je na stránce [Stáhnout](https://www.logicflow-sim.cz).

## Jde LogicFlow používat na tabletu nebo telefonu?

Zatím ne. Webová verze pro tablety a Chromebooky je v plánu po verzi 1.0.

## Smí škola program používat zdarma?

Ano. LogicFlow je pod licencí MIT, takže ho smí kdokoli používat, kopírovat a šířit, i ve škole a ve firmě.

## Našel jsem chybu. Kam ji nahlásit?

Na [GitHub → Issues](https://github.com/SJFidrmuc/LogicFlow/issues). Uveďte verzi programu, systém a přiložte soubor obvodu `.logicflow`, pokud se chyba týká konkrétního obvodu.
