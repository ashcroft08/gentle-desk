# ui-spike: Slint vs iced

Throwaway spikes under `spikes/` (own Cargo workspace, excluded from the main one) to choose the UI framework. Both render the same scripted stream — decoded and reduced by the real `gentle-core` — as a message thread with markdown and streaming (3 chars every 25 ms, two replies).

## Specs

- S1: "spike de UI (hilo de mensajes + markdown + streaming) en Slint vs iced"
- S2: "El framework de UI aún NO está elegido (candidatos: Slint, iced; egui y Dioxus descartados por ahora); se decide con un prototipo."
- S3: UI stays free of I/O and fed by `gentle-core` state (core-m0 S4); the spikes must not change `crates/`.

## Tasks

- [x] T1 (S1, S3) `spikes/common`: scripted RPC stream through decode_line + reduce_chat, 2 tests — inline
- [x] T2 (S1) `spikes/iced` (iced 0.14, `markdown` widget, `Content::push_str`) — inline, 101 lines
- [x] T3 (S1) `spikes/slint` (Slint 1.18, `StyledText`) — inline, 48 lines Rust + 42 lines .slint
- [x] T4 (S2) Compare and recommend — below

## Findings (measured on Windows 11, Rust 1.99.1, debug/release as noted)

| | iced 0.14 | Slint 1.18 |
|---|---|---|
| Markdown in the spike | heading, bold, em, lists, link, inline code, highlighted code block, quote: all render | bold, em, lists, links, inline code render; **headings, code blocks and block quotes are rejected** by `StyledText::from_markdown` (`examples/markdown_support.rs`) |
| Streaming markdown | incremental (`Content::push_str`) | none: whole message re-parsed per delta |
| Release build, cold (incl. deps) | 107 s | 126 s |
| Release binary | 14.8 MB | 13.8 MB |
| Debug build, cold | 65 s | 80 s |
| App code | 101 lines | 90 lines |
| Auto-scroll to bottom | `scrollable.anchor_bottom()` | binding on `content-y` |
| License | MIT | GPL-3.0, or a royalty-free licence with conditions, or commercial (per the SPDX header of its sources) |

Not measured: text selection/copy, IME, accessibility, long-thread scroll performance, CPU use while streaming.

## Recommendation

iced. LLM replies are full of headings, fenced code and quotes; Slint would need our own markdown-to-blocks layer (e.g. pulldown-cmark plus one element per block) to match what iced ships, and its licence is a friction for an MIT project. Build size and time are a wash. Risks to verify next in iced: text selection/copy (core to a chat), long-thread performance, and its churn between versions (0.13 -> 0.14 changed the application API).

## Log

- L1: "dale con el siguiente paso" (UI spike per the M1 brief).
- L2: Reducer does not emit user messages (only assistant); the app layer must append them — the spikes do it in `spike_common::push_user`. To be settled when the `app` crate is built.
- L3: Screenshots confirmed both windows render; the first iced capture was blank only because the debug window was still starting.

## Decision

- L4: "elijo iced" — framework chosen: iced 0.14 (MIT). Slint dropped.
- L5: Selection check, from the iced_widget 0.14.2 sources (not tried by hand in a window): `text`, `rich_text` and the `markdown` widget have no text selection; only `text_editor` and `text_input` do. So rendered assistant messages cannot be selected or copied by dragging. Options: (a) a per-message "Copy" button that copies the raw markdown (cheap, chosen as the starting point), (b) a read-only `text_editor` for selectable plain text (loses markdown styling), (c) wait for upstream support. To revisit if (a) feels insufficient.
