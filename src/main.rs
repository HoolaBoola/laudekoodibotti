use bytes::Bytes;
use carapax::longpoll::LongPoll;
use carapax::methods::GetFile;
use carapax::methods::SendMessage;
use carapax::types::MessageData;
use carapax::types::Update;
use carapax::types::UpdateKind;
use carapax::Api;
use carapax::Dispatcher;
use leptess::LepTess;
use std::env;
use std::io::Write;
use tempfile::NamedTempFile;
use tokio::stream::StreamExt;

#[tokio::main]
async fn main() {
    // Token is given as first command line argument.
    let token = env::args()
        .nth(1)
        .expect("Telegram bot token must be given as an argument!");

    // Setup an API client:
    let api = Api::new(token).expect("Error while connecting to Telegram api!");

    // Dispatcher takes a context which will be passed to each handler
    // we use api client for this, but you can pass any struct.
    let mut dispatcher = Dispatcher::new(api.clone());
    dispatcher.add_handler(handle_update);

    // Poll Telegram server for updates.
    LongPoll::new(api, dispatcher).run().await;
}

#[carapax::handler]
async fn handle_update(context: &Api, input: Update) {
    if let Some(chat_id) = input.get_chat_id() {
        if let UpdateKind::Message(message) = &input.kind {
            /* Stickers and photos are supported. */
            let file_id = match &message.data {
                MessageData::Sticker(sticker) if !sticker.is_animated => Some(&sticker.file_id),
                MessageData::Photo { data, .. } => data.last().map(|p| &p.file_id),
                _ => None,
            };

            if let Some(file_id) = file_id {
                if let Ok(content) = download_file_content(context, file_id).await {
                    /* Create tempfile for the image so it will get deleted later. */
                    if let Ok(mut tempfile) = NamedTempFile::new() {
                        if let Ok(_) = tempfile.write_all(&content) {
                            if let Some(tempfile_path) = tempfile.path().to_str() {
                                if let Ok(text) = read_image(tempfile_path) {
                                    let _ = context.execute(SendMessage::new(chat_id, &text)).await;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

async fn download_file_content(api: &Api, file_id: &str) -> Result<Bytes, ()> {
    if let Ok(file_data) = api.execute(GetFile::new(file_id)).await {
        if let Some(file_path) = file_data.file_path {
            if let Ok(stream) = api.download_file(file_path).await {
                if let Ok(content) = stream.collect::<Result<Bytes, reqwest::Error>>().await {
                    return Ok(content);
                }
            }
        }
    }
    return Err(());
}

fn read_image(file_path: &str) -> Result<String, ()> {
    /* English is probably the best guess for the text of an average sticker. */
    if let Ok(mut reader) = LepTess::new(Some("traineddata"), "eng") {
        reader.set_image(file_path);
        if let Ok(text) = reader.get_utf8_text() {
            return Ok(text);
        }
    }
    return Err(());
}
