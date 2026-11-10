# Third-party material

The copyright line of `LICENSE` names the chain this app stands on, the way
[gentle-shell](https://github.com/Gentleman-Programming/gentle-shell) names pi's
creator in its own MIT license:

- **Mario Zechner (2025)**: pi, the coding agent this app talks to. It speaks pi's
  JSONL RPC and only passes it flags; no pi code is copied here.
- **Gentleman Programming (2026)**: gentle-shell, the workspace built on pi, and
  gentle-shell-desktop, the reference client.
- **ashcroft08 (2026)**: this app, Gentle Shell Desktop.

## gentle-shell-desktop (MIT)

The recorded RPC streams in `crates/gentle-core/tests/fixtures/` were copied from
[gentle-shell-desktop](https://github.com/Gentleman-Programming/gentle-shell-desktop),
and the chat reducer in `crates/gentle-core/src/chat/` re-implements the behaviour
of that project's `chatReducer.ts` (ported and checked against its tests).

    MIT License

    Copyright (c) 2026 Gentleman Programming

    Permission is hereby granted, free of charge, to any person obtaining a copy of
    this software and associated documentation files (the "Software"), to deal in the
    Software without restriction, including without limitation the rights to use, copy,
    modify, merge, publish, distribute, sublicense, and/or sell copies of the Software,
    and to permit persons to whom the Software is furnished to do so, subject to the
    conditions of the MIT License: the above copyright notice and this permission
    notice shall be included in all copies or substantial portions of the Software.
    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND.

The full license text is in that repository's `LICENSE` file.

## Gentle Shell lettering (gentlemanprogramming.com)

The pixel lettering above New session (`apps/desktop/src/brand/mask.ts`) was converted by
`tools/wordmark/import-wordmark.py` from the Gentle Shell wordmark drawn on
[gentlemanprogramming.com](https://gentlemanprogramming.com/), the site of the
Gentleman Programming project that also publishes gentle-shell under the MIT license.
The lettering is that project's artwork.
