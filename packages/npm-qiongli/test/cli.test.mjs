import assert from 'node:assert/strict';
import test from 'node:test';

import { main } from '../lib/cli.mjs';

test('help documents all official subject options', async () => {
  let output = '';
  const exitCode = await main(['help'], {
    stdout: { write: (chunk) => { output += chunk; } },
    stderr: { write: () => {} },
  });

  assert.equal(exitCode, 0);
  assert.match(output, /--subject core\|economics\|accounting\|business\|finance\|economics-accounting/);
});
