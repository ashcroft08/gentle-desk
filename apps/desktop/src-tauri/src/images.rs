//! The images the user attached to a chat: read HERE, once, from the path
//! Tauri's own drop event named, and held until that chat's send takes them.
//!
//! Three rules, and this module is all of them:
//!
//! - The web side sends PATHS and nothing else. It never holds a file's bytes,
//!   because a browser's `File` has no path to give Rust in the first place; the
//!   shell reads the file, checks its extension and its size, and answers with a
//!   name and a preview URL.
//! - A chat's images belong to that chat and to its next send only, so a send
//!   for another chat carries none of them (S5).
//! - The send is where they become pi's own `images` field, with the path left
//!   in the text so the shell's transcript reads the same ([`with_attachments`]).
//!
//! Nothing here touches the user's pi folder (S6): the only files read are the
//! ones the drop named, plus - for a thumbnail - the same file again.

use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use gentle_app::ChatCommand;
use gentle_core::rpc::{base64, ImageContent, RpcCommand};

use crate::contract::ImageView;

/// The most images one message carries (pi's terminal client's own cap).
pub const MAX_IMAGES: usize = 8;

/// The most one message's images may add up to: 64 MB, as pi's terminal client
/// caps it. A file larger than this alone is refused without being read.
pub const MAX_TOTAL: u64 = 64 * 1024 * 1024;

/// The scheme the webview loads a thumbnail from: the shell's own, so a page can
/// never name a file the user did not drop.
pub const SCHEME: &str = "gentle-image";

/// How many attached images keep a servable preview. A very long session drops
/// the oldest thumbnails, which then simply draw nothing.
const MAX_SERVED: usize = 256;

/// The four image types a message can carry, and the MIME type each one travels
/// as in pi's `images` field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mime {
    Png,
    Jpeg,
    Webp,
    Gif,
}

impl Mime {
    /// The type an extension names, case-insensitively. Only these are images a
    /// message may carry; every other file is refused by name.
    pub fn of_extension(extension: &str) -> Option<Self> {
        match extension.to_ascii_lowercase().as_str() {
            "png" => Some(Self::Png),
            "jpg" | "jpeg" => Some(Self::Jpeg),
            "webp" => Some(Self::Webp),
            "gif" => Some(Self::Gif),
            _ => None,
        }
    }

    /// The MIME type pi's `images` field carries.
    pub fn essence(self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::Webp => "image/webp",
            Self::Gif => "image/gif",
        }
    }
}

/// One image a chat holds, already read: the send needs its bytes, the preview
/// needs its path, and the composer needs its name.
#[derive(Debug, Clone)]
pub struct Attachment {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub mime: Mime,
    pub size: u64,
    /// The whole file, base64 encoded: pi's `images` field carries it this way.
    pub data: String,
}

/// One image as the composer and the thread draw it: a name and where the
/// webview loads its thumbnail from, never any of its bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageRef {
    pub id: String,
    pub name: String,
    pub preview_url: String,
}

/// The images of one run, per chat.
#[derive(Debug, Default)]
pub struct Images {
    inner: Mutex<Inner>,
}

#[derive(Debug, Default)]
struct Inner {
    /// The images each chat attached and has not sent yet, in attach order.
    pending: HashMap<String, Vec<Attachment>>,
    /// The images each chat already sent, per message id, oldest message first.
    sent: HashMap<String, VecDeque<SentMessage>>,
    /// Where each attached image is served from. Only ids the user's own drops
    /// created are ever in here: a page cannot name a file.
    served: HashMap<String, Served>,
    /// The served ids in attach order, so the table stays bounded.
    order: VecDeque<String>,
    /// Why the last attach refused something, as `ViewState.images_notice`
    /// carries it. Cleared by the next attach that takes everything.
    notice: Option<String>,
    /// The number the next image id takes. Ids are never reused, so a removed
    /// image's id never names another one.
    next: u64,
}

#[derive(Debug)]
struct SentMessage {
    message: String,
    images: Vec<ImageRef>,
}

#[derive(Debug)]
struct Served {
    path: PathBuf,
    mime: Mime,
}

impl Images {
    /// Attaches the files the drop named to `chat`, refusing what it cannot
    /// take with one plain sentence each. A refusal attaches nothing of that
    /// file; the rest of the drop still arrives.
    ///
    /// The files are read under the lock: a drop is the user's own pace, and the
    /// read is bounded by [`MAX_TOTAL`], so a slice that waits for it waits for
    /// a disk read of at most that.
    pub fn attach(&self, chat: &str, paths: &[String]) {
        if paths.is_empty() {
            return;
        }
        let mut inner = self.inner();
        let mut held = inner.pending.get(chat).map_or(0, Vec::len);
        let mut bytes: u64 = inner
            .pending
            .get(chat)
            .map_or(0, |held| held.iter().map(|image| image.size).sum());
        let mut accepted: Vec<Attachment> = Vec::new();
        let mut refusals: Vec<String> = Vec::new();
        for path in paths {
            let id = inner.next_id();
            match read(path, id, held, bytes) {
                Ok(attachment) => {
                    held += 1;
                    bytes = bytes.saturating_add(attachment.size);
                    accepted.push(attachment);
                }
                Err(reason) => refusals.push(reason),
            }
        }
        for attachment in &accepted {
            inner.serve(attachment);
        }
        if !accepted.is_empty() {
            inner
                .pending
                .entry(chat.to_owned())
                .or_default()
                .extend(accepted);
        }
        inner.notice = (!refusals.is_empty()).then(|| refusals.join(" "));
    }

    /// Takes every image `chat` holds, for the send that is running now.
    pub fn take(&self, chat: &str) -> Vec<Attachment> {
        self.inner().pending.remove(chat).unwrap_or_default()
    }

    /// What `chat` holds, as the composer draws it.
    pub fn pending(&self, chat: &str) -> Vec<ImageRef> {
        self.inner()
            .pending
            .get(chat)
            .map(|held| held.iter().map(image_ref).collect())
            .unwrap_or_default()
    }

    /// Remembers that `message` of `chat` was sent with these images, so the
    /// thread can draw the same thumbnails it showed in the composer.
    pub fn bind(&self, chat: &str, message: &str, attachments: &[Attachment]) {
        if attachments.is_empty() {
            return;
        }
        let mut inner = self.inner();
        let sent = inner.sent.entry(chat.to_owned()).or_default();
        sent.push_back(SentMessage {
            message: message.to_owned(),
            images: attachments.iter().map(image_ref).collect(),
        });
        // Only the messages a push can still carry need their images.
        while sent.len() > crate::view::MESSAGES_PUSHED {
            sent.pop_front();
        }
    }

    /// The images `chat` sent, per message id.
    pub fn sent(&self, chat: &str) -> HashMap<String, Vec<ImageRef>> {
        self.inner()
            .sent
            .get(chat)
            .map(|sent| {
                sent.iter()
                    .map(|entry| (entry.message.clone(), entry.images.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Takes one image of the drop away before it is sent. Answers whether the
    /// id named an image that was waiting.
    pub fn remove(&self, id: &str) -> bool {
        let mut inner = self.inner();
        for held in inner.pending.values_mut() {
            if let Some(at) = held.iter().position(|image| image.id == id) {
                held.remove(at);
                return true;
            }
        }
        false
    }

    /// Why the last attach refused something, as one line.
    pub fn notice(&self) -> Option<String> {
        self.inner().notice.clone()
    }

    /// The bytes of one attached image, for the preview scheme: its type and
    /// what the file holds right now. `None` for an id this run never attached,
    /// so no request can name a file.
    pub fn read(&self, id: &str) -> Option<(&'static str, Vec<u8>)> {
        let (path, mime) = {
            let inner = self.inner();
            let served = inner.served.get(id)?;
            (served.path.clone(), served.mime)
        };
        let bytes = std::fs::read(path).ok()?;
        Some((mime.essence(), bytes))
    }

    fn inner(&self) -> MutexGuard<'_, Inner> {
        self.inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }
}

impl From<ImageRef> for ImageView {
    /// The contract names the same three things the store holds, and nothing of
    /// the file itself.
    fn from(image: ImageRef) -> Self {
        Self {
            id: image.id,
            name: image.name,
            preview_url: image.preview_url,
        }
    }
}

impl Inner {
    /// The id of the next image. Minted even for a file that is refused, so an
    /// id never names two images.
    fn next_id(&mut self) -> String {
        self.next += 1;
        format!("img-{}", self.next)
    }

    /// Records where `attachment`'s preview is served from, dropping the oldest
    /// one when the table is full.
    fn serve(&mut self, attachment: &Attachment) {
        if self.served.len() >= MAX_SERVED {
            if let Some(oldest) = self.order.pop_front() {
                self.served.remove(&oldest);
            }
        }
        self.order.push_back(attachment.id.clone());
        self.served.insert(
            attachment.id.clone(),
            Served {
                path: attachment.path.clone(),
                mime: attachment.mime,
            },
        );
    }
}

/// Where the webview loads one thumbnail from. The form differs per platform
/// (Tauri serves a custom scheme as `http://<scheme>.localhost` on Windows and
/// as `<scheme>://localhost` elsewhere), so the shell builds it and the page
/// only draws what it is given.
pub fn preview_url(id: &str) -> String {
    if cfg!(windows) {
        format!("http://{SCHEME}.localhost/{id}")
    } else {
        format!("{SCHEME}://localhost/{id}")
    }
}

/// The send this command is, with `attachments` merged into it: the text gains
/// the path of every image, so the shell's own transcript reads the same, and
/// pi's `images` field carries the image itself.
///
/// The command is returned untouched when the chat attached nothing, when it is
/// not a send, and when it is a send of something other than a prompt (a dialog
/// answer), so every prompt without images stays byte identical.
pub fn with_attachments(command: ChatCommand, attachments: Vec<Attachment>) -> ChatCommand {
    if attachments.is_empty() {
        return command;
    }
    let ChatCommand::Send { chat, rpc } = command else {
        return command;
    };
    let Some(message) = rpc.prompt_text().map(str::to_owned) else {
        return ChatCommand::Send { chat, rpc };
    };
    let images: Vec<ImageContent> = attachments
        .iter()
        .map(|attachment| ImageContent {
            mime_type: attachment.mime.essence().to_owned(),
            data: attachment.data.clone(),
        })
        .collect();
    ChatCommand::Send {
        chat,
        rpc: RpcCommand::prompt_with_images(prompt_text(&message, &attachments), images),
    }
}

/// What the user typed, and under it one line per image naming its path: the
/// conversation reads the same in the shell's transcript as it does here.
fn prompt_text(message: &str, attachments: &[Attachment]) -> String {
    let lines: Vec<String> = attachments
        .iter()
        .map(|attachment| format!("Attached image: {}", attachment.path.display()))
        .collect();
    format!("{message}\n\n{}", lines.join("\n"))
}

/// One image as the contract draws it.
fn image_ref(attachment: &Attachment) -> ImageRef {
    ImageRef {
        id: attachment.id.clone(),
        name: attachment.name.clone(),
        preview_url: preview_url(&attachment.id),
    }
}

/// Reads and checks one dropped file. `held` and `bytes` are what the chat holds
/// already, so the cap is the message's, never the drop's.
fn read(path: &str, id: String, held: usize, bytes: u64) -> Result<Attachment, String> {
    let path = Path::new(path);
    let name = file_name(path);
    let mime = path
        .extension()
        .and_then(|extension| extension.to_str())
        .and_then(Mime::of_extension)
        .ok_or_else(|| refuse_not_an_image(&name))?;
    let metadata = match std::fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(refuse_missing(&name))
        }
        Err(_) => return Err(refuse_unreadable(&name)),
    };
    if metadata.is_dir() {
        return Err(refuse_folder(&name));
    }
    if held >= MAX_IMAGES {
        return Err(refuse_count(&name));
    }
    let size = metadata.len();
    if size > MAX_TOTAL || bytes.saturating_add(size) > MAX_TOTAL {
        return Err(refuse_weight(&name));
    }
    // Read once, here: the file that disappeared between the metadata and this
    // read is refused like any other it cannot read.
    let contents = std::fs::read(path).map_err(|_| refuse_unreadable(&name))?;
    Ok(Attachment {
        id,
        name,
        path: path.to_path_buf(),
        mime,
        size: contents.len() as u64,
        data: base64::encode(&contents),
    })
}

/// The file's own name, for the sentence and for the thumbnail's text.
fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}

/// The plain sentences a refusal reaches the user with: what was not attached,
/// and why. ASCII, one line each, no jargon.
fn refuse_not_an_image(name: &str) -> String {
    format!("{name} was not attached: only png, jpg, jpeg, webp and gif images can be.")
}

fn refuse_missing(name: &str) -> String {
    format!("{name} was not attached: the file is no longer there.")
}

fn refuse_unreadable(name: &str) -> String {
    format!("{name} was not attached: the file cannot be read.")
}

fn refuse_folder(name: &str) -> String {
    format!("{name} was not attached: it is a folder, not an image file.")
}

fn refuse_count(name: &str) -> String {
    format!("{name} was not attached: a message holds at most {MAX_IMAGES} images.")
}

fn refuse_weight(name: &str) -> String {
    format!(
        "{name} was not attached: a message holds at most {} MB of images.",
        MAX_TOTAL / (1024 * 1024)
    )
}
