import { fileURLToPath } from 'node:url';
import path from 'node:path';

export function packageRoot() {
  return path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
}
