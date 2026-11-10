// The `gentle-agents.activity/v1` widget frame the `helpers` prompt publishes:
// one helper in each interesting state, so the helpers panel can be checked by
// eye. Every helper's thread shows one of the item shapes the thread view draws:
// what it said (markdown), what it thought, its notes, and the tool calls it ran
// with their arguments, output and status - plus a dropped count and an empty
// thread.

const said = (text) => ({ kind: 'text', text });
const thought = (text) => ({ kind: 'thinking', text });
const note = (text) => ({ kind: 'note', text });
const ran = (name, args, output, extra = {}) => ({
  kind: 'tool',
  callId: `call-${name}`,
  name,
  args,
  output,
  ...extra,
});

const thread = (items, dropped = 0) => ({ version: 1, dropped, items });

const task = (id, status, extra = {}, body = thread([{ kind: 'text', text: `Working as ${status}.` }])) => ({
  summary: {
    id,
    agent: 'gentle-ai-explore',
    label: `${status} helper`,
    prompt: `Do the ${status} thing`,
    status,
    createdAt: '2026-09-22T10:00:00.000Z',
    turns: 2,
    toolCalls: 1,
    ...extra,
  },
  thread: body,
});

export function helpersFrame() {
  return {
    schema: 'gentle-agents.activity/v1',
    summary: { running: 1, queued: 1, waiting: 1, finished: 2 },
    tasks: [
      task(
        'r',
        'running',
        { lastStep: 'grep' },
        thread(
          [
            said(
              'Reading the **runtime** crate to list what the bridge must expose, with `Intent` and `ViewState` in mind. Notes: [the task](https://example.com/helper-thread).',
            ),
            thought('Which modules move without an iced import?'),
            ran(
              'read',
              { path: 'crates/runtime/src/lib.rs', limit: 200 },
              '//! The helpers panel as plain text.',
              { running: true },
            ),
            note('TODO: check the helper thread caps'),
          ],
          2,
        ),
      ),
      task('w', 'waiting', {}, thread([said('Waiting for a free slot.')])),
      task('q', 'queued', {}, thread([], 0)),
      task(
        'd',
        'done',
        { endedAt: '2026-09-22T10:05:00.000Z' },
        thread([
          said('All nine contract tests pass against the stand-in shell.'),
          ran('grep', { pattern: 'Intent' }, 'crates/runtime/src/lib.rs:12: pub enum Intent'),
        ]),
      ),
      task(
        'f',
        'failed',
        { endedAt: '2026-09-22T10:06:00.000Z', error: 'model quota exceeded' },
        thread([
          said('The last turn did not finish.'),
          ran('bash', { cmd: 'cargo test -p runtime' }, 'error: test failed', { isError: true }),
        ]),
      ),
    ],
  };
}
