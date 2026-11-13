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
    list_status: { type: 'Complete' },
    sessions_revision: 1,
    total_sessions: 0,
    messages: [],
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
    fatal: null,
    config_notice: null,
    workspace_notice: null,
    sessions_notice: null,
    whats_new: null,
    // A run whose bridge named no version: the About section says so (S5).
    app_version: null,
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
