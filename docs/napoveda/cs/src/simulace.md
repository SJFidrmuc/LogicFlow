# Simulace

Simulace běží neustále, i během kreslení. Každé hradlo má zpoždění jeden takt, takže se signál obvodem šíří postupně, stejně jako ve skutečném obvodu.

## Barvy vodičů

| Barva | Hodnota | Kdy nastane |
| --- | --- | --- |
| světle zelená | 1 | logická jednička |
| tmavě zelená | 0 | logická nula |
| červená | X | neurčitá hodnota: nepřipojený vstup, kolize dvou výstupů, neinicializovaná paměť |
| modrá | Z | nepřipojený vodič (vysoká impedance) |

## Krokování

- **Mezerník** simulaci pozastaví nebo spustí.
- Při pozastavení provede **šipka vpravo** (nebo Simulace → Krok) jeden takt. Je vidět, jak změna prochází hradlo po hradle.
- Pokud se obvod nikdy neustálí (např. invertor zapojený sám do sebe), zobrazí se upozornění **Obvod kmitá**.
- **Simulace → Restartovat simulaci** (Ctrl+R) začne znovu od počátečního stavu.
- Stavový řádek dole ukazuje, zda simulace běží, a číslo taktu.

> **Tip:** Obvody s pamětí, například klopný obvod RS z hradel NOR, začínají ve stavu X. Teprve nastavení (S) nebo nulování (R) určí jejich stav. Po úpravě schématu si takový obvod svůj stav pamatuje dál.
