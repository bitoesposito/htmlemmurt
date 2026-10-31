# htmlemmurt per VS Code

Supporto ai file `.emmurt` di [htmlemmurt](https://github.com/bitoesposito/htmlemmurt), l'HTML in napoletano.

```html
<cristian lingua="it">
  <panz>
    <luccolone>Uè, trase!</luccolone>
    <citofono tipo="buttone" quanno-tuocche="sforna()">Sforna</citofono>
    <fratm>
      facc sforna() {
          stamm a dì("Na margherita!")
      }
    </fratm>
  </panz>
</cristian>
```

## Cosa fa

Gli errori del compilatore compaiono mentre scrivi, in napoletano e con il suggerimento: scrivi `<div>` e ti dice "Vuoi dire `<cascett>`?". Controlla anche i file incollati con `<chiamm>` e il FratmScript dentro `<fratm>`.

Il completamento propone tag, attributi, eventi e valori (`tipo="` → `fattimiei`, `spunta`…), e passando il mouse su un nome vedi l'HTML che diventa.

"Formatta documento" (Maiusc+Alt+F) rimette a posto i rientri come `emmurt fmt`: due spazi per livello, niente spazi in fondo alle righe, al massimo una riga vuota di fila. Su un file che non compila non fa niente, finché non sistemi l'errore.

Ctrl+clic (Cmd+clic su macOS) sul percorso di `<chiamm da="…">` o `<fratm da="…">` apre il file. Un percorso che comincia con `/` parte dalla cartella del progetto.

I colori seguono la struttura della pagina. Dentro `<fratm>` e negli attributi `quanno-*` usa quelli dell'estensione FratmScript se l'hai installata, altrimenti quelli di JavaScript; dentro `<cess>` quelli del CSS.

Gli snippet sono `cristian` (una pagina intera), `fratm`, `fratmda`, `chiamm`, `citofono`, `list` e `modulo`.

## Impostazioni

`htmlemmurt.diagnostics` accende o spegne gli errori mentre scrivi (acceso di default).

## Per compilare

Il compilatore, il plugin Vite e il runtime per il browser stanno nel pacchetto npm [`htmlemmurt`](https://www.npmjs.com/package/htmlemmurt). Vocabolario completo e guida sono nel [README del progetto](https://github.com/bitoesposito/htmlemmurt#readme).

L'icona è ricavata dal [logo HTML5](https://www.w3.org/html/logo/) del W3C, con licenza [CC BY 3.0](https://creativecommons.org/licenses/by/3.0/).
