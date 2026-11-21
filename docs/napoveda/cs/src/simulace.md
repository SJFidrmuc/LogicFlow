# Simulace

Každé hradlo má zpoždění jeden takt, takže se signál obvodem šíří postupně, stejně jako ve skutečném obvodu.

## Režim úprav a režim simulace

Program pracuje ve dvou režimech, podobně jako DigitalWorks:

- **Režim úprav** – schéma kreslíte a upravujete. Logika se přepočítává hned (LED reagují), ale čas stojí: hodiny netikají a časový diagram se neposouvá. Přepínač se přepne **dvojklikem**, tlačítko se stiskne druhým stiskem dvojkliku – jedno kliknutí součástku jen vybere, takže ji nepřepnete omylem při přesouvání.
- **Režim simulace** – spustí ho **F5** (nebo Simulace → Spustit simulaci, nebo mezerník). Schéma je zamčené: nejde přidávat, mazat, přesouvat, otáčet ani kreslit vodiče. Fungují jen ovládací prvky – přepínač a tlačítko jedním kliknutím, posun a zoom pohledu, Tab do podobvodů, časový diagram a pravdivostní tabulka. Ve Vlastnostech jde měnit frekvenci hodin.
- Schéma má v režimu simulace **jantarový rámeček** se štítkem **SIMULACE**, stavový řádek ukazuje Režim úprav / Simulace běží / Simulace pozastavena.
- **Esc** nebo **Shift+F5** simulaci zastaví a vrátí do režimu úprav. **Stav obvodu zůstane** (klopné obvody, čítače, LED i časový diagram); vynuluje ho jen Restartovat simulaci (Ctrl+R).
- Když se v režimu simulace pokusíte o úpravu, objeví se nahoře hláška „Pro úpravy zastavte simulaci (Esc)“.
- Kdo chce upravovat i za běhu, zapne **Simulace → Povolit úpravy za běhu simulace** (výchozí vypnuto).

## Barvy vodičů

| Barva | Hodnota | Kdy nastane |
| --- | --- | --- |
| světle zelená | 1 | logická jednička |
| tmavě zelená | 0 | logická nula |
| červená | X | neurčitá hodnota: nepřipojený vstup, kolize dvou výstupů, neinicializovaná paměť |
| modrá | Z | nepřipojený vodič (vysoká impedance) |

## Krokování

- **Mezerník** v režimu úprav simulaci spustí, v režimu simulace ji pozastaví nebo nechá pokračovat.
- **Šipka vpravo** (nebo Simulace → Krok) provede jeden takt; z režimu úprav přejde rovnou do pozastavené simulace. Je vidět, jak změna prochází hradlo po hradle.
- Pokud se obvod nikdy neustálí (např. invertor zapojený sám do sebe), zobrazí se upozornění **Obvod kmitá**.
- **Simulace → Restartovat simulaci** (Ctrl+R) začne znovu od počátečního stavu.
- Stavový řádek dole ukazuje režim, zda simulace běží, a číslo taktu.

> **Tip:** Obvody s pamětí, například klopný obvod RS z hradel NOR, začínají ve stavu X. Teprve nastavení (S) nebo nulování (R) určí jejich stav. Po úpravě schématu si takový obvod svůj stav pamatuje dál.
