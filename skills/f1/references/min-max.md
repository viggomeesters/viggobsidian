# Min-max

Neem de laatste procenten serieus. Maak eerst zichtbaar welke kosten werkelijk bestaan: bytes, latencyverdeling, geheugen, CPU, wakeups, I/O, energie of menselijke handelingen. Selecteer naar de opdracht; meet geen hele catalogus zonder besliswaarde.

Bewaar een baseline van de juiste release/configuratie. Profileer het relevante pad. Voor een app kunnen startup, leegloop, zoeken en herhaald openen andere oorzaken hebben. Gebruik achtergrondmetingen waar mogelijk; verstorende UI-bediening volgt de bestaande afspraak met de gebruiker.

Formuleer één falsifieerbare hypothese per experiment: oorzaak, wijziging, verwachte winst, mogelijke regressie en beslisregel. Vergelijk dezelfde hardware, versiecondities, data en werklast; onderscheid koud/warm en bron-/debug-/release-/gesigneerde omvang. Wissel meetvolgorde af of controleer drift wanneer dat nodig is. Gebruik genoeg herhalingen voor de verwachte kleine delta.

Rapporteer absolute én relatieve verschillen, spreiding en meetgrens. Gebruik passende percentielen als de staart relevant is, met voldoende waarnemingen. Een verschil binnen variatie blijft inconclusief; neem geen gunstige enkele run als bewijs. “Niet gemeten” is geen nul. Meet energie rechtstreeks voordat je batterijwinst claimt; minder wakeups alleen bewijst die niet.

Behoud bewezen verbetering alleen als de afgesproken eigenschappen overeind blijven. Test het daadwerkelijk gewijzigde gedrag, gegevensbehoud en de geraakte foutpaden. Valt een kandidaat tegen, herstel de vorige variant en bewaar kort de uitkomst zodat dezelfde proef niet zonder nieuwe reden wordt herhaald.

Een kleine winst mag veel moeite kosten bij gekozen min-maxdiepte. Toon inspanning en nieuwe onderhoudslast, maar verwerp haar niet uitsluitend op standaard-ROI. Grote rewrite, nieuwe taal of een dependency verwijderen is een hypothese met kosten, geen vooraf gewonnen optimalisatie.

Werk door tot het afgesproken doel of de afgesproken onderzoeksruimte/begroting is bereikt. Geef open hypotheses en grenzen bij afronding; geen claim dat alle denkbare optimalisaties onderzocht zijn. Borg alleen zinvolle regressiecontroles, geen tests die implementatiedetails spiegelen of ruis als fout behandelen.
