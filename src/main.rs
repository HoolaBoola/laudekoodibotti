use bytes::Bytes;
use carapax::handler;
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
use std::fs::File;
use std::io::Write;
use tokio::stream::StreamExt;

#[tokio::main]
async fn main() {
    // Setup an API client:
    // Token is given as first command line argument.
    let args: Vec<String> = env::args().collect();
    let api = Api::new(&args[1]).unwrap();

    // Dispatcher takes a context which will be passed to each handler
    // we use api client for this, but you can pass any struct.
    let mut dispatcher = Dispatcher::new(api.clone());

    dispatcher.add_handler(handle_update);

    // using long polling
    LongPoll::new(api, dispatcher).run().await;
}

#[handler]
async fn handle_update(context: &Api, input: Update) {
    println!("{:#?}", input);

    if let Some(chat_id) = input.get_chat_id() {
        if let UpdateKind::Message(message) = &input.kind {
            if let MessageData::Sticker(sticker) = &message.data {
                let file_id = &sticker.file_id;

                if let Ok(file_data) = context.execute(GetFile::new(file_id)).await {
                    if let Some(file_path) = &file_data.file_path {
                        if let Ok(stream) = context.download_file(file_path).await {
                            if let Ok(content) =
                                stream.collect::<Result<Bytes, reqwest::Error>>().await
                            {
                                let filename = "foo.webp";

                                if let Ok(mut file) = File::create(filename) {
                                    if let Ok(_) = file.write_all(&content) {
                                        if let Ok(text) = read_image(filename) {
                                            context.execute(SendMessage::new(chat_id, &text)).await;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn read_image(filename: &str) -> Result<String, ()> {
    if let Ok(mut detector) = LepTess::new(Some("traineddata"), "eng") {
        detector.set_image(filename);
        if let Ok(text) = detector.get_utf8_text() {
            return Ok(text);
        }
    }
    return Err(());
}
