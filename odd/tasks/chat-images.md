# Images in the chat (m19 follow-up)

Drag an image from the file manager onto the composer, see it as a preview in the chat, and have the model actually see it. The user's words: "yo queria que en la app se pudiera arrastrar la imagen y se previsualice en el chat pero que internamente se lo envie como texto la ruta"; after finding that the RPC carries images separately they chose: "dale con la 2" (send the image itself, not only its path). In the terminal the user drags the image, the path is pasted, and the file must already be saved (a clipboard screenshot cannot be pasted).

## Specs

- S1 Drag and drop. Dropping one or more image files on the composer attaches them. Only files already saved on disk, only png, jpg, jpeg, webp and gif, at most 8 per message and 64 MB in total (the limits pi's terminal client uses); anything else is refused with a visible reason and nothing is attached.
- S2 Preview. Each attached image shows as a thumbnail with a remove button in the composer, before sending, and the sent user message shows the same thumbnails in the thread. The preview never sends anything by itself.
- S3 What travels. Internally the message carries the path as text, so the conversation reads the same in the shell's own transcript, AND the image itself in pi's own field: the RPC prompt gains `images` with `{"type":"image","data":"<base64>","mimeType":"image/png"}` entries (the shape pi documents: `rpc-types.d.ts` `RpcCommand` `images?: ImageContent[]`, and pi's own error text `{ type: "image", data: <base64>, mimeType }`). The field is omitted when there are no images, so every existing prompt is byte-identical.
- S4 Files are read once, in Rust, from the path the web declares; the web never sends bytes back up. A file that disappeared or cannot be read refuses that attachment with a reason and sends nothing.
- S5 The image is bound to the chat it was attached to: a send for another chat carries no images.
- S6 Nothing here touches the user's pi folder; the only files read are the ones dropped.

## Tasks

- [ ] T1 (S3, S4) Protocol and chat host: `RpcCommand::Prompt` carries images, codec writes the field only when non-empty, round-trip tests, and the host sends them.
- [ ] T2 (S1, S4, S5) Bridge: paths from the web, read and validated in Rust, held per chat until that chat's send, refused with a reason otherwise; contract and bindings.
- [ ] T3 (S2) Web: drop zone on the composer, thumbnails with remove, refusal messages, thumbnails in the user message.
- [ ] T4 (S6, S7) Real-window check with the fake shell (drop, preview, remove, send) and one live check against the real shell that the model sees the image; docs.

## Log

- L1 (2026-11-10, user): "yo queria que en la app se pudiera arrastrar la imagen y se previsualice en el chat pero que internamente se lo envie como texto la ruta. dale prueba".
- L2 (2026-11-10, user): "suspend" no; after the finding that the RPC does not turn a path into an attachment: "dale con la 2" (send the image through pi's `images` field).
- L3 (2026-11-10, agent): the live probe against the real shell in an isolated home never started a turn (131-137 extension_ui_request events, no session file), so the decisive check still waits for a home that answers those requests.
