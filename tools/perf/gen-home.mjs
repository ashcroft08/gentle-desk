// Builds a synthetic pi home with many session files, to measure how the
// session index and the sidebar behave on a big home.
//
//   node tools/perf/gen-home.mjs <home-dir> <sessions> <kb-per-session>
//
// Then point the app at it with `--home <home-dir>` (and a throw-away
// LOCALAPPDATA so the session cache of your real home is left alone).

import fs from 'node:fs';
import path from 'node:path';

const [, , root, countArg, kbArg] = process.argv;
if (!root || !countArg || !kbArg) {
  console.error('usage: node gen-home.mjs <home-dir> <sessions> <kb-per-session>');
  process.exit(2);
}
const count = Number(countArg);
const bytesPerSession = Number(kbArg) * 1024;
const filler = 'x '.repeat(450);

for (let i = 0; i < count; i++) {
  const dir = path.join(root, 'sessions', `--C--proj${i % 12}--`);
  fs.mkdirSync(dir, { recursive: true });
  const header = {
    type: 'session',
    version: 3,
    id: `id${i}`,
    timestamp: '2026-01-01T00:00:00.000Z',
    cwd: `C:/dev/proj${i % 12}`,
  };
  const lines = [JSON.stringify(header)];
  let size = lines[0].length;
  for (let m = 0; size < bytesPerSession; m++) {
    const role = m % 2 ? 'assistant' : 'user';
    const entry = {
      type: 'message',
      id: `m${m}`,
      timestamp: `2026-01-01T00:${String(m % 60).padStart(2, '0')}:00.000Z`,
      message: { role, content: [{ type: 'text', text: `chat ${i} ${role} ${m} ${filler}` }] },
    };
    const line = JSON.stringify(entry);
    lines.push(line);
    size += line.length + 1;
  }
  fs.writeFileSync(path.join(dir, `2026-01-01T00-00-00-000Z_${i}.jsonl`), lines.join('\n') + '\n');
}
console.log(`wrote ${count} sessions of about ${kbArg} KB under ${root}`);
