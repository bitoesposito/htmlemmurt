# Guida a htmlemmurt

Questa guida parte da zero e arriva a un sito pubblicato su GitHub Pages. Servono un terminale e [Node.js](https://nodejs.org) 22 (va bene anche 20.19 o più nuovo). Per l'ultimo passo serve un account GitHub.

## 1. Installa la CLI

Su Linux e macOS:

```bash
curl -fsSL https://raw.githubusercontent.com/bitoesposito/htmlemmurt/main/install.sh | sh
emmurt --version
```

Su Windows scarica lo zip dall'[ultima release](https://github.com/bitoesposito/htmlemmurt/releases/latest) e metti `emmurt.exe` in una cartella del `PATH`.

Se usi VS Code, installa anche l'estensione. Colora le pagine, mostra gli errori mentre scrivi, completa le parole e formatta il file.

```bash
code --install-extension bitoesposito.htmlemmurt
```

## 2. Crea l'app

```bash
emmurt nuovo pizzeria
cd pizzeria
npm install
npm run dev
```

Apri l'indirizzo che stampa Vite, di solito http://localhost:5173. Nella cartella `pizzeria` trovi:

| File | Cosa contiene |
| --- | --- |
| `index.emmurt` | La pagina |
| `pezzi/cappiell.emmurt`, `pezzi/pere.emmurt` | Intestazione e piè di pagina, incollati nella pagina |
| `src/main.fratm` | Quello che fa la pagina, in FratmScript |
| `stile.css` | Lo stile |
| `vite.config.js` | Vite con i plugin di htmlemmurt e FratmScript |

## 3. La prima pagina

Apri `index.emmurt`. Una pagina ridotta all'osso è così:

```html
<cristian lingua="it">
  <capa>
    <titolo>'A pizzeria</titolo>
  </capa>
  <panz>
    <luccolone>Uè, trase!</luccolone>
    <chiacchier>Oggi sfurnammo <fort>pizze fritte</fort>.</chiacchier>
  </panz>
</cristian>
```

`<cristian>` è `<html>`, `<capa>` è `<head>`, `<panz>` è `<body>`. Un file che comincia con `<cristian>` è una pagina: il compilatore ci aggiunge `<!DOCTYPE html>` e `<meta charset="utf-8">`. Cambia il testo e salva, il browser si ricarica da solo.

## 4. Le parole

Elementi, attributi, eventi e valori fissi hanno il loro nome napoletano:

```html
<cascett classe="menu">
  <cartulin da="margherita.jpg" spiegazione="Na margherita">
  <miett tipo="fattimiei" nomme="password" esempio="'A password">
  <vattenn addo="menu.html">'O menù</vattenn>
</cascett>
```

Se scrivi il nome inglese, l'errore ti dice quello giusto:

```text
✗ index.emmurt:3:6: <div> nun è napoletano.
  Vuoi dire <cascett>?
```

Trovi tutte le parole con `emmurt vocabolario`, nella [tabella del README](../README.md#vocabolario) e nel completamento dell'estensione. I nomi HTML che non hanno ancora una parola passano come sono. Classi, id, testi e indirizzi restano come li scrivi, e lo stesso vale per gli attributi `data-*` e `aria-*`.

I tag si chiudono sempre, nell'ordine in cui li hai aperti. Fanno eccezione quelli vuoti, come `<acap>`, `<miett>` e `<cartulin>`.

## 5. Lo stile

Lo stile dell'app sta in `stile.css`, collegato nella `<capa>` con `<attacca rapporto="stile" addo="/stile.css">`. È CSS normale. Dentro `<cess>`, e nell'attributo `cess`, lo puoi scrivere anche in napoletano:

```html
<cess>
  citofono:passanno {
    culore: russo;
  }
</cess>
```

diventa `button:hover { color: red; }`.

## 6. Quello che fa la pagina

Il JavaScript si scrive in [FratmScript](https://github.com/kkratterf/fratmscript), JavaScript con le parole napoletane. L'app carica `src/main.fratm` con `<fratm da="/src/main.fratm" />`. Per le cose piccole puoi scrivere direttamente nella pagina:

```html
<citofono tipo="buttone" quanno-tuocche="sforna()">Sforna</citofono>
<fratm>
  tien pizze = 0
  facc sforna() {
      pizze++
      stamm a dì(pizze)
  }
</fratm>
```

`quanno-tuocche` è `onclick`. Negli attributi `quanno-*` e dentro `<fratm>` ci va FratmScript, che diventa JavaScript. Le parole di FratmScript stanno nella [sua guida](https://github.com/kkratterf/fratmscript#readme).

## 7. Pezzi riusati

`pezzi/cappiell.emmurt` non comincia con `<cristian>`, quindi è un pezzo. La pagina lo incolla con `<chiamm da="pezzi/cappiell.emmurt" />`. Il percorso è relativo al file che chiama; se comincia con `/` parte dalla cartella del progetto.

Un pezzo può ricevere parametri e contenuto. Salva questo come `pezzi/carta.emmurt`:

```html
<fattariello classe="carta">
  <luccolo>{{titolo}}</luccolo>
  {{cuntenuto}}
</fattariello>
```

e chiamalo dalla pagina:

```html
<chiamm da="pezzi/carta.emmurt" titolo="Margherita">
  <chiacchier>Pummarola, mozzarella e vasenicola.</chiacchier>
</chiamm>
```

`titolo` riempie `{{titolo}}`, e quello che sta tra `<chiamm>` e `</chiamm>` va al posto di `{{cuntenuto}}`. Un parametro che manca o che il pezzo non usa è un errore.

## 8. Un'altra pagina

Crea `menu.emmurt` accanto a `index.emmurt`, che cominci con `<cristian>`. Vite la serve come `/menu.html` e la mette nella build. Per collegare le pagine usa indirizzi relativi, come `<vattenn addo="menu.html">`: così funzionano anche su GitHub Pages, dove il sito sta in una sottocartella.

## 9. Controlla e formatta

```bash
emmurt check .
emmurt fmt .
```

`emmurt check` compila pagine e pezzi senza scrivere niente. `emmurt fmt` rimette a posto rientri e righe vuote e non tocca nient'altro. In VS Code gli errori compaiono mentre scrivi e "Formatta documento" fa come `emmurt fmt`.

## 10. Pubblica su GitHub Pages

GitHub Pages serve file statici, quindi va bene per un'app di sole pagine come questa. Un'app fatta con `emmurt nuovo --completa` ha anche un server Node, e quella va su un servizio che lo fa girare.

In `vite.config.js` c'è `base: './'`: gli indirizzi della build sono relativi, e il sito funziona anche in `utente.github.io/pizzeria/`.

Crea `.github/workflows/pagine.yml`:

```yaml
name: Pagine

on:
  push:
    branches: [main]
  workflow_dispatch:

permissions:
  contents: read
  pages: write
  id-token: write

concurrency:
  group: pages
  cancel-in-progress: true

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v7
      - uses: actions/setup-node@v7
        with:
          node-version: 22
      - run: npm ci
      - run: npm run build
      - uses: actions/upload-pages-artifact@v5
        with:
          path: dist

  deploy:
    needs: build
    runs-on: ubuntu-latest
    environment:
      name: github-pages
      url: ${{ steps.deploy.outputs.page_url }}
    steps:
      - id: deploy
        uses: actions/deploy-pages@v5
```

Poi:

1. Crea un repository vuoto su GitHub, per esempio `pizzeria`.
2. Nelle impostazioni del repository, sotto Settings → Pages, scegli "GitHub Actions" come Source.
3. Carica il progetto. `package-lock.json` va nel commit, perché `npm ci` lo usa.

   ```bash
   git init
   git add .
   git commit -m "prima versione"
   git branch -M main
   git remote add origin https://github.com/utente/pizzeria.git
   git push -u origin main
   ```

Il workflow parte a ogni push su `main`. Quando finisce, il sito è su `https://utente.github.io/pizzeria/` e l'indirizzo compare anche in Settings → Pages.

## E poi

- Le regole complete e il vocabolario stanno nel [README](../README.md).
- `emmurt nuovo pizzeria --completa` crea un'app con un'API e un database SQLite.
- Nel [playground](https://bitoesposito.github.io/htmlemmurt/) provi una pagina senza installare niente.
