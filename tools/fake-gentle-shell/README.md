# fake-gentle-shell

A stand-in for `gentle-shell --mode rpc`, for testing the clients (today
`gentle-term`, later the GUI) without a model, network or real session files.
Plain Node, no dependencies, no build step.

`fake.mjs` reads one JSON command per line on stdin, writes pi's JSONL rpc
events on stdout, and exits 0 when stdin ends. Event shapes follow the recorded
streams in `crates/gentle-core/tests/fixtures/`.

## Use

Point the client at it and pass a prompt:

```sh
export GENTLE_SHELL_BIN="$PWD/tools/fake-gentle-shell/fake.mjs"
cargo run -p gentle-term -- hola
# echo: hola
```

The launcher resolves a `.mjs` file to `node <file> --mode rpc`, so `node` must
be on `PATH`.

## Prompts

What a turn does depends on the prompt text:

| prompt     | behaviour                                                                                                                       |
| ---------- | ------------------------------------------------------------------------------------------------------------------------------- |
| `error`    | assistant message ending with `stopReason: "error"` (`quota exceeded (fake)`), then a `notify` warning; the turn settles normally |
| `dialog`   | a `confirm` (`d1`), a `select` (`d2`, options `a`/`b`) and an `input` (`d3`), one at a time; the reply echoes the raw responses    |
| `helpers` | publishes a helpers widget with one helper per state (running, waiting, queued, done, failed), then echoes |
| `slow`     | a `text_delta` of `tick ` every 100 ms until `{"type":"abort"}`; the message then ends with `ABORTED`                             |
| anything   | `echo: <prompt>`                                                                                                                |

Every turn starts by acknowledging the prompt with
`{"type":"response","command":"prompt","success":true,"data":{"disposition":"started"}}`.

`slow` self-terminates after 30 s with the same `ABORTED` message, so a client
that forgets to abort cannot keep the process alive.

## Tests

`crates/term/tests/end_to_end.rs` drives the built `gentle-term` binary against
this script, which keeps the tests deterministic and free:

```sh
cargo test -p gentle-term
```
