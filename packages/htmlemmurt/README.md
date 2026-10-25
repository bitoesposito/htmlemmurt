# htmlemmurt

Il compilatore di [htmlemmurt](https://github.com/bitoesposito/htmlemmurt) (HTML in napoletano) in WebAssembly, con un plugin Vite e un runtime per il browser.

```bash
npm i -D htmlemmurt
```

## Da Node

```js
import { compile, compileFratm, formatError } from 'htmlemmurt';

const { html, dependencies, errors } = compile('<cascett classe="a">Uè</cascett>', {
  filename: 'index.emmurt',
});
// html → '<div class="a">Uè</div>'
```

`compile` non lancia mai eccezioni: gli errori stanno in `errors`, con `message`, `file`, `line`, `column` e `suggestion`, in napoletano. Le opzioni sono `filename`, `root` (la cartella per i percorsi che cominciano con `/`), `read` (per leggere i file in un altro modo) e `inline` (incolla nella pagina i file di `<fratm da>`). `compileFratm` compila un modulo FratmScript.

## Con Vite

```js
// vite.config.js
import htmlemmurt from 'htmlemmurt/vite';

export default { plugins: [htmlemmurt()] };
```

Ogni `x.emmurt` che comincia con `<cristian>` diventa `x.html`, in sviluppo e in build. Anche i moduli `.fratm` passano dal plugin.

## Nel browser

```html
<script type="module" src="https://cdn.jsdelivr.net/npm/htmlemmurt@0.1/browser.js"></script>
<script type="text/htmlemmurt" src="index.emmurt"></script>
```

Da JavaScript: `import { init, compile } from 'htmlemmurt/web'`, poi `await init()`.
