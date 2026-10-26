# htmlemmurt

HTML, ma comme si deve 🤌

Provalo nel [playground](https://bitoesposito.github.io/htmlemmurt/): scrivi a sinistra, a destra vedi l'HTML e la pagina vera.

htmlemmurt è HTML in napoletano. Scrivi `<cascett>` e il compilatore scrive `<div>`. È il fratello di [FratmScript](https://github.com/kkratterf/fratmscript): i blocchi `<fratm>` e gli attributi `quanno-*` sono FratmScript e diventano JavaScript. Il compilatore è in Rust e gira anche in Node, in Vite e nel browser grazie a WebAssembly.

```html
<cristian lengua="it">
  <capa>
    <titulo>'A Pizzeria</titulo>
  </capa>
  <panz>
    <chiamm da="pezzi/cappiell.emmurt" />
    <luccolone>Uè, trase!</luccolone>
    <citofono tipo="buttone" quanno-tuocche="sforna()">Sforna</citofono>
    <fratm>
      tien pizze = 0
      facc sforna() {
          pizze++
          stamm a dì(`Sfurnate: ${pizze}`)
      }
    </fratm>
  </panz>
</cristian>
```

diventa

```html
<!DOCTYPE html>
<html lang="it">
  <head>
    <meta charset="utf-8">
    <title>'A Pizzeria</title>
  </head>
  <body>
    <header>…</header>
    <h1>Uè, trase!</h1>
    <button type="button" onclick="sforna();">Sforna</button>
    <script>
let pizze = 0;
function sforna() {
  pizze++;
  console.log(`Sfurnate: ${pizze}`);
}
</script>
  </body>
</html>
```

## Struttura

```
htmlemmurt/
├── crates/
│   ├── emmurt-core/       # compilatore: lexer, controlli, traduzione, include
│   ├── emmurt-cli/        # il comando emmurt
│   └── emmurt-wasm/       # binding WebAssembly
├── packages/
│   ├── htmlemmurt/        # pacchetto npm: API, plugin Vite, runtime per il browser
│   ├── playground/        # il playground web, scritto in htmlemmurt e FratmScript
│   └── vscode-extension/  # estensione VS Code
├── examples/              # pagine d'esempio e l'app pizzeria (Vite + FratmScript)
└── e2e/                   # test in Chromium
```

## Le regole

Il compilatore accetta solo napoletano. `<div>` è un errore, e l'errore ti dice cosa scrivere al posto suo:

```
✗ index.emmurt:3:6: <div> nun è napoletano.
     3 |     <div classe="menu">
       |      ^
  Vuoi dire <cascett>?
```

Lo stesso vale per attributi (`class` → `classe`), eventi (`onclick` → `quanno-tuocche`) e valori fissi (`tipo="password"` → `tipo="fattimiei"`). Passano così come sono gli attributi `data-*` e `aria-*` e gli elementi con un trattino nel nome, come `<pizza-card>`, perché sono tuoi.

I tag si chiudono sempre e nell'ordine giusto, a parte quelli vuoti come `<acap>` e `<miett>`. `<cascett />` vale `<cascett></cascett>`.

Un file che comincia con `<cristian>` è una pagina: il compilatore ci mette `<!DOCTYPE html>` e, se manca, `<meta charset="utf-8">`. Gli altri file sono pezzi da incollare con `<chiamm da="pezzi/menu.emmurt" />`. Il percorso è relativo al file che chiama; se comincia con `/` parte dalla cartella del progetto.

`<fratm>` contiene FratmScript e diventa `<script>`. Se usa `chiamm` o `mann for` diventa `<script type="module">`. Un blocco classico rende globali le sue funzioni, così gli attributi `quanno-*` le possono chiamare. `<fratm da="main.fratm" />` dentro Vite resta un modulo, e Vite compila il file e quello che importa; con la CLI e nel browser il codice viene incollato nella pagina, quindi lì il file non può importare altri `.fratm`.

Per il JavaScript normale c'è `<copione>`, per il CSS `<cess>`. Cess sarà anche il nome del CSS in napoletano, che arriverà come progetto a parte.

## Con Vite

Un'app vera si scrive con Vite. Per partire da un'app già pronta, con una pagina, un pezzo condiviso e un modulo FratmScript, c'è `emmurt nuovo pizzeria` (vedi la CLI più sotto). Il plugin serve e compila ogni `x.emmurt` come `x.html` e compila i moduli `.fratm`:

```js
// vite.config.js
import htmlemmurt from 'htmlemmurt/vite';

export default {
  plugins: [htmlemmurt()],
};
```

Le pagine sono i file `.emmurt` che cominciano con `<cristian>`, e la build le include tutte. In sviluppo una pagina con un errore mostra il messaggio in napoletano e si ricarica quando salvi. Il plugin compila già i `.fratm`, quindi non serve aggiungere anche `fratmscript/vite`.

```bash
npm i -D htmlemmurt vite
```

[`examples/pizzeria`](examples/pizzeria) è un'app completa: due pagine, pezzi condivisi, un carrello in FratmScript diviso in moduli.

## La CLI

Su Linux e macOS:

```bash
curl -fsSL https://raw.githubusercontent.com/bitoesposito/htmlemmurt/main/install.sh | sh
```

Lo script scarica il binario giusto dall'[ultima release](https://github.com/bitoesposito/htmlemmurt/releases/latest), controlla lo SHA-256 e lo mette in `~/.local/bin` (`EMMURT_INSTALL_DIR` cambia la cartella, `EMMURT_VERSION=v0.1.0` sceglie la versione). Su Windows c'è lo zip nella release. Con Rust installato va bene anche:

```bash
cargo install --git https://github.com/bitoesposito/htmlemmurt emmurt-cli
```

```bash
emmurt nuovo pizzeria                # un'app Vite pronta, con htmlemmurt e FratmScript
emmurt build index.emmurt            # scrive index.html accanto
emmurt build sito/ --out dist/       # tutte le pagine di una cartella
emmurt check sito/                   # solo controllo, anche dei pezzi
emmurt watch sito/ --out dist/       # ricompila quando cambia qualcosa
emmurt vocabolario                   # tutte le parole e il loro HTML
```

`--json` stampa un solo oggetto JSON con gli errori (file, riga, colonna, suggerimento), utile per editor e agenti. `-` legge da stdin e `--stdout` stampa l'HTML.

## Nel browser, senza build

```html
<script type="module" src="https://cdn.jsdelivr.net/npm/htmlemmurt@0.1/browser.js"></script>
<script type="text/htmlemmurt" src="index.emmurt"></script>
```

Ogni `<script type="text/htmlemmurt">`, inline o con `src`, viene compilato e prende il posto del tag. Una pagina intera sostituisce il documento. Gli errori compaiono nella pagina e nella console. Serve un server (anche `python3 -m http.server`), perché il browser deve scaricare i file. Esempio in [`examples/senza-build`](examples/senza-build).

## VS Code

L'estensione sta sul [Marketplace di VS Code](https://marketplace.visualstudio.com/items?itemName=bitoesposito.htmlemmurt): cerca "htmlemmurt" tra le estensioni, oppure

```bash
code --install-extension bitoesposito.htmlemmurt
```

Colora i file `.emmurt` e mostra gli errori mentre scrivi. Completa tag, attributi, eventi e valori, e passando il mouse su un nome vedi il suo HTML. Dentro `<fratm>` usa i colori dell'estensione FratmScript, se l'hai installata.

## Vocabolario

`emmurt vocabolario` stampa le stesse tabelle.

<details>
<summary>Elementi</summary>

| htmlemmurt | HTML |
|---|---|
| `<cristian>` | `<html>` |
| `<capa>` | `<head>` |
| `<panz>` | `<body>` |
| `<titulo>` | `<title>` |
| `<inciucc>` | `<meta>` |
| `<attacca>` | `<link>` |
| `<basa>` | `<base>` |
| `<cess>` | `<style>` |
| `<fratm>` | `<script>` con FratmScript compilato |
| `<copione>` | `<script>` con JavaScript così com'è |
| `<senzafratm>` | `<noscript>` |
| `<cappiell>` | `<header>` |
| `<pere>` | `<footer>` |
| `<tomtom>` | `<nav>` |
| `<core>` | `<main>` |
| `<piezz>` | `<section>` |
| `<fattariello>` | `<article>` |
| `<allato>` | `<aside>` |
| `<indirizzo>` | `<address>` |
| `<luccolone>` | `<h1>` |
| `<luccolo>` | `<h2>` |
| `<luccoliello>` | `<h3>` |
| `<luccolillo>` | `<h4>` |
| `<luccolino>` | `<h5>` |
| `<luccolicchio>` | `<h6>` |
| `<cascett>` | `<div>` |
| `<chiacchier>` | `<p>` |
| `<stacc>` | `<hr>` |
| `<accussì>` / `<accussi>` | `<pre>` |
| `<citazione>` | `<blockquote>` |
| `<list>` | `<ul>` |
| `<elenco>` | `<ol>` |
| `<voce>` | `<li>` |
| `<vocabolario>` | `<dl>` |
| `<parola>` | `<dt>` |
| `<significato>` | `<dd>` |
| `<quadro>` | `<figure>` |
| `<didascalia>` | `<figcaption>` |
| `<vattenn>` | `<a>` |
| `<scatulell>` | `<span>` |
| `<acap>` | `<br>` |
| `<fort>` | `<strong>` |
| `<calcato>` | `<em>` |
| `<guapp>` | `<b>` |
| `<stuorto>` | `<i>` |
| `<sottolineato>` | `<u>` |
| `<cancellato>` | `<s>` |
| `<piccirill>` | `<small>` |
| `<evidenziato>` | `<mark>` |
| `<codice>` | `<code>` |
| `<fonte>` | `<cite>` |
| `<virgulette>` | `<q>` |
| `<abbreviazione>` | `<abbr>` |
| `<tiempo>` | `<time>` |
| `<sotto>` | `<sub>` |
| `<ncoppa>` | `<sup>` |
| `<tastiera>` | `<kbd>` |
| `<levato>` | `<del>` |
| `<aggiunto>` | `<ins>` |
| `<cartulin>` | `<img>` |
| `<cornice>` | `<picture>` |
| `<funtana>` | `<source>` |
| `<filmato>` | `<video>` |
| `<suono>` | `<audio>` |
| `<fenestella>` | `<iframe>` |
| `<tela>` | `<canvas>` |
| `<disegno>` | `<svg>` |
| `<tavulat>` | `<table>` |
| `<cartello>` | `<caption>` |
| `<capotavula>` | `<thead>` |
| `<cuorpotavula>` | `<tbody>` |
| `<peretavula>` | `<tfoot>` |
| `<fila>` | `<tr>` |
| `<casella>` | `<td>` |
| `<capocasella>` | `<th>` |
| `<modulo>` | `<form>` |
| `<miett>` | `<input>` |
| `<citofono>` | `<button>` |
| `<targhett>` | `<label>` |
| `<scegli>` | `<select>` |
| `<scelta>` | `<option>` |
| `<gruppo>` | `<optgroup>` |
| `<ragionamento>` | `<textarea>` |
| `<recinto>` | `<fieldset>` |
| `<leggenda>` | `<legend>` |
| `<suggerimenti>` | `<datalist>` |
| `<risultato>` | `<output>` |
| `<avanzamento>` | `<progress>` |
| `<misura>` | `<meter>` |
| `<spoiler>` | `<details>` |
| `<anteprim>` | `<summary>` |
| `<dialogo>` | `<dialog>` |
| `<stampo>` | `<template>` |
| `<fessura>` | `<slot>` |

`<chiamm da="…" />` non diventa un tag: al suo posto il compilatore incolla il file.

</details>

<details>
<summary>Attributi</summary>

Un attributo con `ncopp'a` vale solo su quegli elementi; gli altri valgono ovunque.

| htmlemmurt | HTML | Valori |
|---|---|---|
| `addo` (ncopp'a `<modulo>`) | `action` |  |
| `tipo` (ncopp'a `<miett>`) | `type` | `testo` → `text`, `fattimiei` → `password`, `segreto` → `password`, `posta` → `email`, `numero` → `number`, `spunta` → `checkbox`, `pallino` → `radio`, `manna` → `submit`, `azzera` → `reset`, `buttone` → `button`, `carta` → `file`, `data` → `date`, `ora` → `time`, `dataeora` → `datetime-local`, `mese` → `month`, `settimana` → `week`, `colore` → `color`, `cursore` → `range`, `cerca` → `search`, `telefono` → `tel`, `indirizzo` → `url`, `annascuso` → `hidden`, `cartulin` → `image` |
| `tipo` (ncopp'a `<citofono>`) | `type` | `manna` → `submit`, `azzera` → `reset`, `buttone` → `button` |
| `tipo` (ncopp'a `<fratm>`) | `type` | `modulo` → `module` |
| `classe` | `class` |  |
| `chi` | `id` |  |
| `cess` | `style` |  |
| `titulo` | `title` |  |
| `lengua` | `lang` |  |
| `annascuso` | `hidden` |  |
| `turno` | `tabindex` |  |
| `ruolo` | `role` |  |
| `scrivibile` | `contenteditable` | `overo` → `true`, `sfòls` → `false`, `sfols` → `false` |
| `trascinabile` | `draggable` | `overo` → `true`, `sfòls` → `false`, `sfols` → `false` |
| `correggi` | `spellcheck` | `overo` → `true`, `sfòls` → `false`, `sfols` → `false` |
| `addo` | `href` |  |
| `da` | `src` |  |
| `spiegazione` | `alt` |  |
| `larghezza` | `width` |  |
| `altezza` | `height` |  |
| `arape` | `target` | `nuova` → `_blank`, `ccà` → `_self`, `cca` → `_self`, `pate` → `_parent`, `ncoppa` → `_top` |
| `rapporto` | `rel` | `stile` → `stylesheet`, `icona` → `icon`, `manifesto` → `manifest`, `precarica` → `preload` |
| `scarica` | `download` |  |
| `tipo` | `type` |  |
| `codifica` | `charset` |  |
| `nomme` | `name` |  |
| `cuntenuto` | `content` |  |
| `carica` | `loading` | `chianochiano` → `lazy`, `subbeto` → `eager` |
| `aropp` | `defer` |  |
| `ampressa` | `async` |  |
| `valore` | `value` |  |
| `esempio` | `placeholder` |  |
| `stutato` | `disabled` |  |
| `spuntato` | `checked` |  |
| `scigliuto` | `selected` |  |
| `poffozza` | `required` |  |
| `nunsetocca` | `readonly` |  |
| `assaje` | `multiple` |  |
| `subbeto` | `autofocus` |  |
| `minimo` | `min` |  |
| `massimo` | `max` |  |
| `passo` | `step` |  |
| `lunghezzaminima` | `minlength` |  |
| `lunghezzamassima` | `maxlength` |  |
| `modello` | `pattern` |  |
| `pe` | `for` |  |
| `suggerimenti` | `list` |  |
| `accetta` | `accept` |  |
| `etichetta` | `label` |  |
| `righe` | `rows` |  |
| `colonne` | `cols` |  |
| `metodo` | `method` | `piglia` → `get`, `manna` → `post`, `dialogo` → `dialog` |
| `ricorda` | `autocomplete` | `sì` → `on`, `si` → `on`, `no` → `off` |
| `allarga` | `colspan` |  |
| `allonga` | `rowspan` |  |
| `partesubbeto` | `autoplay` |  |
| `comandi` | `controls` |  |
| `ripete` | `loop` |  |
| `muto` | `muted` |  |
| `copertina` | `poster` |  |
| `quanno` | `datetime` |  |
| `araputo` | `open` |  |

</details>

<details>
<summary>Eventi</summary>

Dentro le virgolette ci va FratmScript.

| htmlemmurt | HTML |
|---|---|
| `quanno-tuocche` | `onclick` |
| `quanno-tuocche-doje-vote` | `ondblclick` |
| `quanno-scrive` | `oninput` |
| `quanno-cagne` | `onchange` |
| `quanno-manne` | `onsubmit` |
| `quanno-carica` | `onload` |
| `quanno-premi` | `onkeydown` |
| `quanno-aize` | `onkeyup` |
| `quanno-trase` | `onmouseenter` |
| `quanno-esce` | `onmouseleave` |
| `quanno-piglie` | `onfocus` |
| `quanno-perde` | `onblur` |

</details>

## Sviluppo

Le regole per chi contribuisce, persone o agenti, stanno in [`AGENTS.md`](AGENTS.md).

```bash
cargo test --workspace                                       # compilatore, golden test
cd packages/htmlemmurt && npm run build && npm test          # WebAssembly e pacchetto npm
cd packages/vscode-extension && npm run compiler && npm test # estensione
cd examples/pizzeria && npm install && cd ../../e2e && npm install && npm test  # Chromium
```

## Licenza

MIT
