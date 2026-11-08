# backlog: what is left, in the order it will be done

Ordered by (1) shipped-but-unverified first, (2) risk of user-visible problems, (3) dependencies, (4) new features, (5) housekeeping. Each batch = one branch + one pull request, checked by CI before `main`.

1. **DONE** — **GUI checks of shipped-but-unverified paths** (feat/gui-checks, `gui-checks-m7.md`): model-error box, extension dialogs (confirm/select/input/editor), Stop button; fix what breaks. Uses a stand-in gentle-shell, no model cost.
2. **DONE** — **Visual polish that blocks readability**: helper card colours/contrast, `done` colour, timestamps in the sidebar.
3. **DONE (refresh off the UI thread + persisted cache)** — **Session list performance, as one batch**: refresh off the UI thread, then persist the index cache across runs (the second depends on the first's shape).
4. **DONE (no optimization needed; see perf-check-m11.md)** — **Long-thread performance check** (measure first; optimize only if it hurts).
5. **Rename / delete sessions** (needs two decisions: how pi stores a name, and a confirmation step before deleting a file).
6. **Housekeeping** (chat search is DONE, see session-search-m12.md): tests for `gentle-term` argument parsing, markdown in helper thread items, bump the `actions/checkout` major version, project filter in the sidebar.

## Log

- L1: "ordena lo que toca hacer y dale" — order above; starting with 1.
7. **DONE (virtual-sidebar-m13.md)** — **Virtualized sidebar** (step 2 of session-list-design.md): fixed-height rows, windowed from `scrollable::on_scroll`; prototype first.
8. **Data-level window for the thread** (step 3): last N messages + "load earlier", markdown parsed only for the window.
9. **`SessionStore` / SQLite** (step 4): only if the user wants to search inside message text or a home passes ~100 000 chats.
10. **DONE (refresh-watcher-m16.md)** — **Refresh strategy that does not scale with the number of files** (found in virtual-sidebar-m13.md): at 50 000 chats the 5 s polling costs ~30 % of a core. File-system watcher or cost-bounded polling; decide with measurements.
11. **DONE (bounded-title-m14.md)** — **Bounded session title in the summary and the cache** (found in virtual-sidebar-m13.md): the whole first message is kept in memory and persisted (65 MB cache at 50 000 chats).
12. **DONE (sessions-loading-m17.md)** — **Loading state for the first full pass** (found in refresh-watcher-m16.md; design and tasks in `sessions-loading-m17.md`): the sidebar claims there are no chats while a big home is being read for the first time; show a loading indicator and partial results.
13. **Workspaces + independent chats** (sidebar grouped by cwd; chats without a folder as independent group; absorbs the project filter of item 6; needs its own design doc first).
