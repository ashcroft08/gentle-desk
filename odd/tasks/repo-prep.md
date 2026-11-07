# repo-prep: get the project ready to publish

## Specs

- S1: "en que momento seria bueno armar un repo para subir?" and, offered "Preparar el repo local primero": prepare locally; the user creates the remote and pushes.
- S2: Nothing is pushed or created on GitHub by the agent.

## Tasks

- [x] T1 `main` branch at the tip of the linear history (17 commits + this one); the feature branches are its ancestors and were left in place
- [x] T2 `LICENSE` (MIT, holder `ashcroft08` as chosen by the user: "dejar solo ashcroft08") and `NOTICE.md` crediting the reference app (MIT, Gentleman Programming) for the copied fixtures and the ported reducer behaviour
- [x] T3 `.github/workflows/ci.yml`: windows-latest, fmt --check, clippy -D warnings, test (the same three commands pass locally; the workflow itself has not run — it needs a remote)
- [x] T4 Review of what would become public: no secrets (matches were token-usage fields and `%dp0%`); three lines in `odd/` name the local path `C:\Users\josep\...`; commit metadata carries the git author `ashcroft08 <johangracia40@gmail.com>`

## Open decisions for the user

- Remote name/visibility and whether to publish the commit email (GitHub offers a noreply address).
- Keep `spikes/` (records the Slint/iced decision; the Slint spike depends on a GPL/royalty-free-licensed crate, only in its own workspace) or drop it before publishing.
- Delete the merged `feat/*` and `refactor/*` branches locally, or keep them as history markers.

## Log

- L1: "si, en que momento seria bueno armar un repo para subir?" -> recommendation: now, before the helpers (everything lived only on this machine, 7 stacked branches, no main, no license, no CI).
