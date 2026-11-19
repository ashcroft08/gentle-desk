# composer-dock: the status row, the chips and file attachments

The maintainer saw two things in the other desktop app and asked for them on
gentle-desk's own components: the row that shows what the agent is doing while it
works, and the composer's chips (context, folder, branch, mode, attachments). This
document exists BEFORE any code so the data question is answered first: three of
those chips are cheap, one needs a new command, and one cannot be built at all.

## Specs (verbatim)

- S1: "quiero agregar mas cosas de diseño hay podemos? Podemos mostrar el contexto
  en ese diseño y mira como se muestra cuando piensa y el workin, tambien lo de la
  sesion si esta normal/YOLO/Super yolo".
- S2: "tambien quiero hacer esto de que muestre en que workspace estamos, solo el
  nombre de la carpeta no toda la direccion, tambien la rama en la que estamos si es
  que hay github si no no se muestra nada, quiero implementar este mismo diseño
  usando nuestros componentes y colores".
- S3: "hay como un clip que abre el explorador de archivos para insertar archivos,
  al final ya tenemos lo de arrastrar imagenes, quiero hacer que tambien se pueda
  con archivos, como al final lo que se manda es la direccion".

## What the data says (read-only exploration, 2026-11-18)

| Chip | Status | Note |
| --- | --- | --- |
| Context percent + bar | AVAILABLE | `ContextUsageView` already carries percent AND window |
| Working state + elapsed | NEEDS PLUMBING | the mark exists; no timestamp reaches the view |
| Folder (last segment) | NEEDS PLUMBING | `SessionItem::project()` exists per session; the foreground chat's is not in the view |
| Git branch | NOT AVAILABLE | nothing reads git today; needs a command |
| Permission mode | AVAILABLE, unwired | over `/gentle:desktop` (`get_permission_mode`); read is safe, WRITE is unverified and upstream refuses YOLO outside a TUI |
| Output tokens / cost | NOT AVAILABLE | pi's RPC sends neither; drawing them would be inventing a number |

## Rulings taken from that table

- R1: the mode is drawn as a READ-ONLY label, never a switch. Upstream refuses to
  grant YOLO outside a TUI on purpose, so a control here would be a lie (same
  criterion as the "two methods" ruling for profiles: the terminal keeps what only
  the terminal can do).
- R2: no token or cost chip. Not "later": not at all, until pi exposes it.
- R3: with no repo, the branch chip is NOT drawn - no gap, no dash, no "detached".
- R4: the folder shows the LAST SEGMENT only; the full path lives in the tooltip.
- R5: an attached file from outside the workspace says so, because the child may not
  be able to read that path.
- R6: the clip inserts the file's PATH. Images keep travelling as images; a file
  travels as a path the agent opens with its own tools.

## Tasks

- [ ] T1 (S1, S2) the status row: the working state with an elapsed time and the
  project name (last segment only), sharing ONE anatomy with the reasoning row.
- [ ] T2 (S1, S2) the composer chips: context percent with its bar (reusing what
  exists), the folder name, and the branch - the branch behind a new Tauri command
  that answers "no repo" rather than failing.
- [ ] T3 (S1) the permission mode, read-only, behind `supports()`.
- [ ] T4 (S3) attachments: a picker command plus the existing drag-and-drop, chips
  in the composer, the outside-workspace warning, and paths inserted into the
  message.

## Log

- L1 (2026-11-18, the maintainer's words in full, one line each of S1-S3 above).
- L2 (2026-11-18, exploration): the table above is the answer to "can we"; it is why
  this document rules out tokens and a mode switch instead of drawing them.
- L3 (2026-11-18, design first): the artboard is
  `design/composer-dock/01-dock.html`. Nothing compiles until it is approved.
