use bytes::Bytes;
use carapax::methods::GetFile;
use carapax::methods::SendMessage;
use carapax::methods::SetWebhook;
use carapax::types::MessageData;
use carapax::types::Update;
use carapax::types::UpdateKind;
use carapax::webhook;
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

    // Heroku will fill these in.
    let heroku_app_name =
        env::var("HEROKU_APP_NAME").expect("HEROKU_APP_NAME env variable not found!");
    let port = env::var("PORT")
        .expect("PORT env variable not found!")
        .parse::<u16>()
        .unwrap();

    // Setup an API client:
    let api = Api::new(&token).expect("Error while connecting to Telegram api!");

    // Dispatcher takes a context which will be passed to each handler
    // we use api client for this, but you can pass any struct.
    let mut dispatcher = Dispatcher::new(api.clone());
    dispatcher.add_handler(handle_update);

    // Tell Telegram that a server is running.
    api.execute(SetWebhook::new(format!(
        "https://{}.herokuapp.com/{}",
        heroku_app_name, &token
    )))
    .await
    .expect("Error while setting webhook!");

    // Run a webserver where Telegram can report new updates.
    webhook::run_server(([0, 0, 0, 0], port), format!("/{}", &token), dispatcher)
        .await
        .expect("Error while running webhook server!");
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
