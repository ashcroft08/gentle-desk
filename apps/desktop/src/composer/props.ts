// The composer's props, in their own type: the docs of every prop live here so
// the component stays within its line budget. Nothing but types.
import type {
  BranchView,
  ChatActivityMark,
  ContextUsageView_Serialize as ContextUsageView,
  FolderView,
  ImageView,
  ModelChipView,
} from '../contract/bindings'

/** The composer's props (each doc says who owns the value it carries). */
export type ComposerProps = {
  /** The foreground chat: which draft is shown, and where a send goes. */
  chat: string | null
  /** Its mark, `null` when it is idle; it decides Send, Stop and the reason. */
  mark?: ChatActivityMark | null
  /** The images the chat holds, as Rust pushes them: a name, a preview URL and
   * the id the remove names back. The page never holds a byte of an image. */
  images?: ImageView[]
  /** Why the last drop was refused, as one plain line, or `null`. */
  imagesNotice?: string | null
  /** What the model chip shows, as Rust pushes it, or `null`. */
  chip?: ModelChipView | null
  /** How full this chat's context is (0.1.12, S5), as Rust pushes it. */
  contextUsage?: ContextUsageView | null
  /** The folder this chat runs in (design/composer-dock S2, R4): the last
   * segment of its working directory and the whole path, or `null` for a chat
   * whose session is not listed yet - absent, never a placeholder. */
  folder?: FolderView | null
  /** The branch that folder is on (design/composer-dock S2, R3, R4), as Rust
   * pushes it: the name and the directory it was read from, `null` whenever
   * there is no branch to draw. `null` draws no chip at all - not a gap, not a
   * placeholder (R3). */
  branch?: BranchView | null
}
