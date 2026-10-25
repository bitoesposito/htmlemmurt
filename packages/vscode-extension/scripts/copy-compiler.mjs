// Copies the Node build of the compiler (packages/htmlemmurt, `npm run build`
// there first) into compiler/, so the extension ships it.
import { copyFileSync, mkdirSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = dirname(dirname(fileURLToPath(import.meta.url)));
const from = join(root, '..', 'htmlemmurt', 'wasm', 'node');
mkdirSync(join(root, 'compiler'), { recursive: true });
for (const file of ['emmurt_wasm.js', 'emmurt_wasm_bg.wasm']) {
  copyFileSync(join(from, file), join(root, 'compiler', file));
}
copyFileSync(join(root, '..', '..', 'LICENSE'), join(root, 'LICENSE'));
