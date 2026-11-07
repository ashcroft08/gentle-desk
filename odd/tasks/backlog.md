# backlog: what is left, in the order it will be done

Ordered by (1) shipped-but-unverified first, (2) risk of user-visible problems, (3) dependencies, (4) new features, (5) housekeeping. Each batch = one branch + one pull request, checked by CI before `main`.

1. **DONE** — **GUI checks of shipped-but-unverified paths** (feat/gui-checks, `gui-checks-m7.md`): model-error box, extension dialogs (confirm/select/input/editor), Stop button; fix what breaks. Uses a stand-in gentle-shell, no model cost.
2. **Visual polish that blocks readability**: helper card colours/contrast, `done` colour, timestamps in the sidebar.
3. **Session list performance, as one batch**: refresh off the UI thread, then persist the index cache across runs (the second depends on the first's shape).
4. **Long-thread performance check** (measure first; optimize only if it hurts).
5. **Rename / delete sessions** (needs two decisions: how pi stores a name, and a confirmation step before deleting a file).
6. **Housekeeping**: tests for `gentle-term` argument parsing, markdown in helper thread items, bump the `actions/checkout` major version, project filter in the sidebar.

## Log

- L1: "ordena lo que toca hacer y dale" — order above; starting with 1.
