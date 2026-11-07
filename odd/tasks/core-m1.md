# core-m1: backend process layer and terminal end-to-end

Continues core-m0. M1 = a `backend` crate that launches the real `gentle-shell --mode rpc`, and a terminal binary proving prompt -> streamed reply end to end. UI framework spike comes after, in its own feature document.

## Specs

- S1: "lanzar gentle-shell por RPC desde Rust" — `backend` spawns `gentle-shell --mode rpc`; it does not reimplement agent logic (core-m0 S3).
- S2: "En Windows el global es `gentle-shell.cmd`, que NO se puede ejecutar sin shell (Node daba EINVAL en la app original); resolverlo bien (cmd /c o el .mjs real) y cubrirlo con tests."
- S3: "Alimentar LineSplitter con los bytes de stdout, nunca matar procesos huérfanos al cerrar." (read as: closing never leaves orphan processes behind)
- S4: "Binario de terminal de punta a punta: prompt -> respuesta en streaming usando decode_line + reduce_chat, contra el gentle-shell real."
- S5: "Mostrar errores del modelo (créditos, rate limit) de forma visible, no como un mensaje normal."
- S6: Layers one-way: backend depends on gentle-core; gentle-core stays pure (core-m0 S4). Use the global gentle-shell 4.6.0, not the dev checkout.
- S7: Out of this feature (next): Slint vs iced spike, setWidget helpers, incremental session index.

## Tasks

- [x] T1 (S1, S2, S6) `backend` crate: launcher resolution (GENTLE_SHELL_BIN, PATH lookup, .cmd shim -> node + real .mjs, cmd /c fallback), tested — inline — RED not observed (impl and tests written together); 10 tests, clippy/fmt clean — commit: see git log (feat/core-m1)
- [x] T2 (S1, S3) `backend` process: spawn, stdout bytes -> LineSplitter -> channel, stdin commands, stderr drained, graceful shutdown with no orphans — inline — RED not observed (impl first); 18 backend tests drive real node children; mutation check: without the taskkill /T step the orphan test hangs/fails (grandchild must be spawned detached, libuv otherwise kills it via a job object); clippy/fmt clean — commit: see git log (feat/core-m1)
- [ ] T3 (S4, S5) terminal binary + visible model errors in the chat state — inline — commit: pending

## Log

- L1: Request (summary of the user's M1 brief): see S1-S7 above, quoted from it. Bugs of the reference app not to repeat: sidebar loaded once, "New chat" no-op by reference equality, `.cmd` spawn without shell, `listAll()` parsing every session file.
