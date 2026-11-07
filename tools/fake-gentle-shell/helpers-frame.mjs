// The `gentle-agents.activity/v1` widget frame the `helpers` prompt publishes:
// one helper in each interesting state, so the helpers panel can be checked by eye.

const task = (id, status, extra = {}) => ({
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
  thread: { version: 1, dropped: 0, items: [{ kind: 'text', text: `Working as ${status}.` }] },
});

export function helpersFrame() {
  return {
    schema: 'gentle-agents.activity/v1',
    summary: { running: 1, queued: 1, waiting: 1, finished: 2 },
    tasks: [
      task('r', 'running', { lastStep: 'grep' }),
      task('w', 'waiting'),
      task('q', 'queued'),
      task('d', 'done', { endedAt: '2026-09-22T10:05:00.000Z' }),
      task('f', 'failed', { endedAt: '2026-09-22T10:06:00.000Z', error: 'model quota exceeded' }),
    ],
  };
}
