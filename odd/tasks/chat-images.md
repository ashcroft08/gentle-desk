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

- [x] T1 (S3, S4) Protocol: `RpcCommand::PromptWithImages`, the `images` key written only when something is attached, pi's block shape `{type, data, mimeType}`, base64 in `crates/gentle-core/src/rpc/base64.rs`. Route: writer; commit: b8078c7 (#77).
- [x] T2 (S1, S4, S5) Bridge: paths from the web, read and validated in Rust (`apps/desktop/src-tauri/src/images.rs`: 8 images, 64 MB, png/jpg/jpeg/webp/gif), held per chat, merged into that chat's prompt in `effects/runtime.rs` beside `PendingFolders`, never through `crates/app`. Route: writer; commit: b8078c7.
- [x] T3 (S2) Web: Tauri's drop event (`composer/drop.ts`), thumbnails with a remove button, refusals said out loud, the same thumbnails in the sent message. Route: writer; commit: b8078c7, polished in #78.
- [x] T4 (S6, S7) Real-window check and the end-to-end proof: the user dropped two saved images in the app and the model answered describing them; the session file shows the user turn with a text part and an image part (`image/png`, 201420 base64 bytes), so the picture travelled, not only its path. Commit: b8078c7.

### Known limits

- A real drag can only be done with a mouse: the agent's automation cannot drag a file onto the window, so the drop path is covered by the web tests and this real-window run by the user, not by an automated window check.
- Thumbnails past 256 attached images in one very long session stop being served (`MAX_SERVED`), and those draw nothing.
- The image is read once per attachment; dropping the same file twice attaches it twice.

## Log

- L1 (2026-11-10, user): "yo queria que en la app se pudiera arrastrar la imagen y se previsualice en el chat pero que internamente se lo envie como texto la ruta. dale prueba".
- L2 (2026-11-10, user): "suspend" no; after the finding that the RPC does not turn a path into an attachment: "dale con la 2" (send the image through pi's `images` field).
- L3 (2026-11-10, agent): the live probe against the real shell in an isolated home never started a turn (131-137 extension_ui_request events, no session file), so the decisive check still waits for a home that answers those requests.
- L4 (2026-11-10, CI): the first CI run of the images failed only at `npm run test`, in `App.tabs.test.ts`, because the composer's drop effect asked Tauri for a webview the test does not have; the full suite had passed locally only because another file left a global stub behind. Fixed in a08ad49 with a try/catch that answers a working unsubscribe when there is nothing to listen to, and a test that fails without it.
- L5 (2026-11-10, user): "si me respondio mandando la imagen" plus the design review of the attachments row; the polish (whole picture, smaller remove inside the corner, file name on hover) went to #78.
- L6 (2026-11-10, agent): the writer given this feature was cancelled at 120 turns with the tree green; the parent finished the verification and committed. Too wide a surface for one writer: the lesson is one slice per launch, as the delegation card says.
