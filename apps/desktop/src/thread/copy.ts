// The thread's words, in one place (the sidebar's `copy.ts` convention): the
// two role lines a screen reader hears before a message, the Copy label, the
// one live status line and the model-error title. `crates/ui/src/view/thread.rs`
// draws a cross mark before "Model error"; no glyph is drawn here, because the
// thread draws no icon characters at all.

/** The accessible name of the scroll region. */
export const THREAD_LABEL = 'Chat thread'

/** The visually hidden author line of a user message's article. */
export const YOU_SAID = 'You said'

/** The same for the assistant's. */
export const ASSISTANT_SAID = 'Assistant said'

/** The one message action this slice draws. */
export const COPY = 'Copy'

/** The only live region of the thread, present while a message streams. */
export const REPLYING_STATUS = 'Assistant is replying'

/** The dimension of the danger box, unchanged from the iced thread. */
export const MODEL_ERROR_TITLE = 'Model error'
