# AGENTS.md

Regole per agenti (Claude, Codex, Copilot, Cursor…) e persone che lavorano su htmlemmurt. Se una regola qui va contro un'istruzione del maintainer, vince il maintainer.

## Cos'è il repo

htmlemmurt è HTML in napoletano: un compilatore severo che traduce nomi napoletani di elementi, attributi e valori in HTML standard. I blocchi `<fratm>` e gli attributi `quanno-*` sono FratmScript, compilato con `fratm-core` (dipendenza git, tag fissato in `Cargo.toml`); `<cess>` e `cess="…"` sono CSS in napoletano, tradotto dallo `stile` di `fratm-core`. Le parole sono quelle di `fratm_core::html`, le stesse del JSX di FratmScript.

| Percorso | Cosa contiene |
| --- | --- |
| `crates/emmurt-core/src/vocabolario.rs` | Le ricerche nel vocabolario di `fratm_core::html` e quello che serve solo alle pagine (`<chiamm>`, elementi vuoti e grezzi) |
| `crates/emmurt-core/src/lexer.rs` | Testo, commenti, tag e contenuto grezzo (`<fratm>`, `<cess>`…) |
| `crates/emmurt-core/src/lib.rs` | Compilatore: controlli, traduzione, include, `<fratm>`, errori |
| `crates/emmurt-cli` | Il comando `emmurt` (`build`, `check`, `watch`, `vocabolario`) |
| `crates/emmurt-wasm` | Binding WebAssembly usati da npm ed estensione |
| `packages/htmlemmurt` | Pacchetto npm: API Node, plugin Vite, runtime browser (`wasm/` è generato) |
| `packages/vscode-extension` | Grammatica, snippet, completamento, diagnostica (`compiler/` è generato) |
| `packages/playground` | Playground web su GitHub Pages, scritto in htmlemmurt e FratmScript con il plugin Vite |
| `crates/emmurt-cli/src/nuovo/` | Il modello dell'app di `emmurt nuovo` |
| `examples/*.emmurt` | Pagine d'esempio con snapshot golden |
| `examples/pizzeria` | App Vite con FratmScript, provata in Chromium da `e2e/` |

## Lingua

- Codice e commenti in inglese.
- Parole del linguaggio, messaggi del compilatore ed esempi in napoletano: sono il prodotto.
- README, commit e issue in italiano.

## Build e test

Gli stessi controlli della CI (`.github/workflows/ci.yml`):

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build -p emmurt-cli && ./target/debug/emmurt check examples/*.emmurt examples/pezzi && ./target/debug/emmurt check examples/pizzeria
cd packages/htmlemmurt && npm run build && npm test           # serve wasm-pack
cd packages/vscode-extension && npm run compiler && npm test
cd packages/playground && npm install && npm run build      # dopo il WebAssembly
cd examples/pizzeria && npm install && cd ../../e2e && npm install && npm test   # Chromium
```

Dopo una modifica al compilatore, `npm run build` in `packages/htmlemmurt` rigenera il WebAssembly: plugin Vite, runtime e test e2e usano quello.

## Regole per le modifiche

- La CI deve essere verde. Non saltare, cancellare o indebolire un test per farla passare, e non zittire clippy con un `allow` generico.
- Ogni correzione del compilatore arriva con un test che fallisce prima e passa dopo, accanto al codice, e controlla l'HTML generato.
- I golden test (`crates/emmurt-core/tests/golden.rs`) fotografano l'output di ogni `examples/*.emmurt` e di ogni `tests/golden/cases/*.emmurt`; i casi `*.sbagliato.emmurt` fotografano l'errore. Un nuovo messaggio d'errore porta un caso nuovo. Dopo una modifica voluta: `EMMURT_UPDATE_GOLDEN=1 cargo test -p emmurt-core --test golden`, poi rileggi il diff.
- Il vocabolario è condiviso con FratmScript: una parola nuova o cambiata si propone a kkratterf/fratmscript in `crates/fratm-core/src/html.rs`, poi si aggiorna il tag di `fratm-core` e la tabella del README (un test lo controlla). L'estensione legge il vocabolario dal WebAssembly e non va toccata.
- Non cambiare il significato di una parola esistente senza una issue che lo chieda: i file `.emmurt` già scritti devono continuare a compilare.
- Il compilatore resta severo: niente ripieghi silenziosi sull'HTML inglese.
- Un limite di FratmScript non si aggira qui dentro: si segnala a kkratterf/fratmscript con issue e PR.
- Niente dipendenze nuove senza un motivo scritto nella PR.

## Documentazione e backlog

- Niente `PLAN.md`, `ROADMAP.md` o note di progetto nel repo. Documentazione ammessa: i README, i commenti nel codice, questo file, `CLAUDE.md` e `docs/neapoletan-dictionary.md`, il dizionario concordato con FratmScript (cambia insieme a `fratm-core/src/html.rs`).
- Il backlog sta nelle issue di GitHub.

## Commit e PR

- Una sola riga, [Conventional Commits](https://www.conventionalcommits.org/) in italiano: `tipo(scope): messaggio`, minuscolo, senza punto finale. Niente corpo, niente trailer `Co-Authored-By` né altri riferimenti a strumenti.
- Tipi: `feat`, `fix`, `perf`, `refactor`, `test`, `docs`, `ci`, `build`, `chore`.
- Scope: `core`, `cli`, `wasm`, `npm`, `vite`, `browser`, `vscode`, `examples`, `e2e`, `ci`.
- Una PR, un argomento. Il titolo della PR è una riga di commit.

## Rilasci

SemVer. La versione sta in `Cargo.toml` (`[workspace.package]`), `packages/htmlemmurt/package.json` e `packages/vscode-extension/package.json`: si cambiano insieme. Fa eccezione l'estensione, che sale da sola quando cambia solo lei (grammatica, snippet, completamento). Un cambio solo di icone non alza la versione: esce col rilascio successivo.
