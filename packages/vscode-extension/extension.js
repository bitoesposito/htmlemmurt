// VS Code glue: errors while you type, completion, hover, formatting and links for .emmurt files.
'use strict';

const fs = require('node:fs');
const path = require('node:path');
const vscode = require('vscode');
const wasm = require('./compiler/emmurt_wasm.js');
const lingua = require('./lib/lingua.js');

const vocab = JSON.parse(wasm.vocabulary());

// A path that starts with / is from the project folder: the nearest folder
// above the page that has the file, or else the workspace folder
function fromProject(doc, p) {
  const top = vscode.workspace.getWorkspaceFolder(doc.uri)?.uri.fsPath;
  for (let dir = path.dirname(doc.uri.fsPath); ; dir = path.dirname(dir)) {
    const target = path.join(dir, p);
    if (fs.existsSync(target) || dir === top || dir === path.dirname(dir)) return target;
  }
}

function activate(context) {
  const diagnostics = vscode.languages.createDiagnosticCollection('htmlemmurt');
  const timers = new Map();

  const check = (doc) => {
    if (doc.languageId !== 'htmlemmurt') return;
    if (!vscode.workspace.getConfiguration('htmlemmurt').get('diagnostics')) {
      diagnostics.delete(doc.uri);
      return;
    }
    const file = doc.uri.fsPath.split(path.sep).join('/');
    const read = (p) => {
      const target = p.startsWith('/') && !fs.existsSync(p) ? fromProject(doc, p) : p;
      try {
        return fs.readFileSync(target, 'utf8');
      } catch {
        return null;
      }
    };
    const { errors } = wasm.compile(doc.getText(), file, read, true);
    diagnostics.set(
      doc.uri,
      errors.map((error) => {
        const place = lingua.errorPlace(error, file);
        const start = new vscode.Position(place.line, place.column);
        const range = doc.getWordRangeAtPosition(start) ?? new vscode.Range(start, start.translate(0, 1));
        const diagnostic = new vscode.Diagnostic(range, place.message, vscode.DiagnosticSeverity.Error);
        diagnostic.source = 'htmlemmurt';
        return diagnostic;
      }),
    );
  };

  const later = (doc) => {
    clearTimeout(timers.get(doc.uri.toString()));
    timers.set(doc.uri.toString(), setTimeout(() => check(doc), 250));
  };

  const completion = {
    provideCompletionItems(doc, position) {
      const from = new vscode.Position(Math.max(0, position.line - 30), 0);
      const where = lingua.context(doc.getText(new vscode.Range(from, position)));
      if (!where) return [];
      const items =
        where.kind === 'tag' || where.kind === 'closing'
          ? lingua.tagItems(vocab)
          : where.kind === 'attr'
            ? lingua.attrItems(vocab, where.tag)
            : lingua.valueItems(vocab, where.tag, where.attr);
      const kind =
        where.kind === 'value' ? vscode.CompletionItemKind.EnumMember : vscode.CompletionItemKind.Property;
      return items.map((item) => {
        const out = new vscode.CompletionItem(item.label, kind);
        out.detail = item.detail;
        return out;
      });
    },
  };

  const hover = {
    provideHover(doc, position) {
      const range = doc.getWordRangeAtPosition(position, /[\p{L}\p{N}_-]+/u);
      if (!range) return null;
      const lines = lingua.meaning(vocab, doc.getText(range));
      return lines.length ? new vscode.Hover(new vscode.MarkdownString().appendCodeblock(lines.join('\n'), 'text'), range) : null;
    },
  };

  // The whole document, or nothing when it does not compile: the errors say why
  const formatter = {
    provideDocumentFormattingEdits(doc) {
      const text = wasm.format(doc.getText());
      if (text === undefined || text === doc.getText()) return [];
      return [vscode.TextEdit.replace(new vscode.Range(doc.positionAt(0), doc.positionAt(doc.getText().length)), text)];
    },
  };

  // Ctrl+click on da="…" of <chiamm> and <fratm> opens the file
  const linker = {
    provideDocumentLinks(doc) {
      return lingua.links(doc.getText()).map((link) => {
        const target = link.path.startsWith('/')
          ? fromProject(doc, link.path)
          : path.join(path.dirname(doc.uri.fsPath), link.path);
        const range = new vscode.Range(doc.positionAt(link.start), doc.positionAt(link.end));
        return new vscode.DocumentLink(range, vscode.Uri.file(target));
      });
    },
  };

  context.subscriptions.push(
    diagnostics,
    vscode.languages.registerDocumentLinkProvider('htmlemmurt', linker),
    vscode.languages.registerDocumentFormattingEditProvider('htmlemmurt', formatter),
    vscode.languages.registerCompletionItemProvider('htmlemmurt', completion, '<', ' ', '"', '/'),
    vscode.languages.registerHoverProvider('htmlemmurt', hover),
    vscode.workspace.onDidOpenTextDocument(check),
    vscode.workspace.onDidSaveTextDocument(check),
    vscode.workspace.onDidChangeTextDocument((e) => later(e.document)),
    vscode.workspace.onDidCloseTextDocument((doc) => diagnostics.delete(doc.uri)),
    vscode.workspace.onDidChangeConfiguration((e) => {
      if (e.affectsConfiguration('htmlemmurt')) vscode.workspace.textDocuments.forEach(check);
    }),
  );
  vscode.workspace.textDocuments.forEach(check);
}

module.exports = { activate };
