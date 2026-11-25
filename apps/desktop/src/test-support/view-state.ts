// The view state a test pushes, as Rust pushes it: the default fields a
// test does not override, split from tauri-mock so no file grows past its
// lint cap. Imported through tauri-mock's re-export.
import type {
  ChatActivityView,
  DialogView_Serialize,
  NoticeView_Serialize,
  SessionTreeRow_Serialize,
  SessionsWindow_Serialize,
  ViewState_Serialize,
  WorkspaceRowView_Serialize,
} from '../contract/bindings'

/** A view state as Rust pushes it, with the fields a test cares about set. */
export function viewState(overrides: Partial<ViewState_Serialize> = {}): ViewState_Serialize {
  return {
    theme: 'system',
    // No preference stored yet: the web starts from the system's own locale.
    language: null,
    // Nothing was turned off: a chat that needs the user says so while the
    // window is not in front (notifications S4).
    notifications: 'when-unfocused',
    // No pi found, nothing chosen, nothing asked.
    shell: {
      pi_path: null,
      choice: null,
      ask: false,
      launcher_found: true,
      install: { type: 'Idle' },
      restart_needed: false,
    },
    foreground_chat: null,
    foreground_title: null,
    // A chat whose session is not listed yet has no folder to name (S2, R4).
    foreground_folder: null,
    // And no branch on it either: nothing read this run's git yet, and a folder
    // no repository holds never has one (S2, R3).
    foreground_branch: null,
    list_status: { type: 'Complete' },
    sessions_revision: 1,
    total_sessions: 0,
    // No chat open: the Updates pane's gate is free (S4).
    open_chats: 0,
    messages: [],
    // No chat open, so the transcript holds no row either.
    items: [],
    // And no row was read from a session file: nothing is provisional.
    transcript_provisional: false,
    notices: [],
    dialog: null,
    helpers: null,
    selected_helper: null,
    helper_thread: null,
    activity: [],
    images: [],
    images_notice: null,
    model_chip: {
      status: { type: 'Unknown' },
      models: [],
      current: null,
      thinking_level: null,
      thinking_levels: [],
    },
    // A chat that has not answered yet: every field null, never a zero.
    context_usage: { tokens: null, percent: null, window: null },
    // Nothing read this chat's profile yet: the Profile tab (T2c) reads it when
    // it opens, and until an answer lands it says it is reading.
    chat_profile: null,
    // No chat answered for its session mode yet: with no mode there is no
    // control to draw (composer-dock T3b, L7).
    session_mode: null,
    // The Settings panes before their first read (0.1.12): nothing known yet.
    auth: { status: { type: 'Unknown' }, providers: [] },
    updates: {
      status: 'Unknown',
      // `skipped` is absent while nothing is skipped: the field is the config's
      // own, read by the pane and the gear's dot (0.8.0 settings-polish S1), and
      // the push leaves it out until the user skips a version.
      desk: { current: null, latest: null, update_available: null },
      pi: { current: null, latest: null, update_available: null },
      shell: { current: null, latest: null, update_available: null },

      packages: { available: false, behind: [], unknown: false, truncated: false, unchecked: false, checked: null },
    },
    // Nothing asked the shell's usage sources anything yet: the pane says it is
    // reading and asks for its one read when the section is opened.
    usage: { status: { type: 'Unknown' }, providers: [] },
    fatal: null,
    config_notice: null,
    workspace_notice: null,
    sessions_notice: null,
    whats_new: null,
    // A run whose bridge named no version: the About section says so (S5).
    app_version: null,
    // No chat is in the foreground in a test's default slice, but view A's data
    // is the state's own (welcome-screen T1): a test that draws it sets it.
    welcome: null,
    ...overrides,
  }
}

/** One activity entry, as `ViewState.activity` carries it. */
export function activity(chatId: string, mark: ChatActivityView['activity']): ChatActivityView {
  return { chat_id: chatId, activity: mark }
}

/** The notice a chat's death leaves: an error that offers the one action. */
export function restartNotice(id: number, text: string, chatId: string): NoticeView_Serialize {
  return {
    id,
    text,
    level: 'Error',
    action: { type: 'RestartChat', data: { chat_id: chatId } },
  }
}

/** The confirm dialog a `confirm` extension request leaves. */
export function confirmDialog(
  id: string,
  title = 'Allow this command?',
  message = 'Run the command?',
): DialogView_Serialize {
  return { kind: 'Confirm', id, title, message }
}

/** The select dialog a `select` extension request leaves. */
export function selectDialog(
  id: string,
  options: string[] = ['a', 'b'],
  title = 'Pick a letter',
): DialogView_Serialize {
  return { kind: 'Select', id, title, options }
}

/** The input dialog an `input` extension request leaves. */
export function inputDialog(
  id: string,
  placeholder: string | null = 'name',
  title = 'Your name',
): DialogView_Serialize {
  return { kind: 'Input', id, title, placeholder }
}

/** The editor dialog an `editor` extension request leaves. */
export function editorDialog(
  id: string,
  prefill: string | null = null,
  title = 'Edit the note',
): DialogView_Serialize {
  return { kind: 'Editor', id, title, prefill }
}

/** A notice that only reports something, with nothing to fix (T7's to draw). */
export function plainNotice(
  id: number,
  text: string,
  level: NoticeView_Serialize['level'] = 'Warning',
): NoticeView_Serialize {
  return { id, text, level, action: null }
}

/** One chat row, as `sessions_window` answers it under its workspace header. */
export function chatRow(
  id: string,
  overrides: Partial<Omit<Extract<SessionTreeRow_Serialize, { kind: 'Chat' }>, 'kind'>> = {},
): SessionTreeRow_Serialize {
  return {
    kind: 'Chat',
    id,
    title: id,
    project: 'gentle-desk',
    message_count: 1,
    modified_ago: 'now',
    ...overrides,
  }
}

/** One workspace header row, as `sessions_window` answers it. */
export function workspaceRow(
  label: string,
  overrides: Partial<Omit<WorkspaceRowView_Serialize, 'label'>> = {},
): SessionTreeRow_Serialize {
  return {
    kind: 'Workspace',
    key: label.toLowerCase(),
    label,
    path: `C:\\dev\\${label}`,
    name: label,
    collapsed: false,
    chat_count: 0,
    available: true,
    reason: null,
    ...overrides,
  }
}

/** One window page, as `sessions_window` answers it. */
export function sessionsWindow(total: number, rows: SessionTreeRow_Serialize[]): SessionsWindow_Serialize {
  return { total, rows }
}
