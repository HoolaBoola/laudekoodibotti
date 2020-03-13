use carapax::longpoll::LongPoll;
use carapax::methods::GetFile;
use carapax::methods::SendMessage;
use carapax::types::MessageData;
use carapax::types::Update;
use carapax::types::UpdateKind;
use carapax::Dispatcher;
use carapax::{async_trait, ExecuteError, Handler};
use carapax::{handler, types::Command};
use carapax::{types::Message, HandlerResult};
use carapax::{Api, Config};
use carapax::{ErrorHandler, ErrorPolicy, HandlerError, LoggingErrorHandler};
use cmd_lib::CmdResult;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::*;
use std::os;
use std::path::Path;
use std::process;
use tokio::net::{TcpListener, TcpStream};
use tokio::stream::{Stream, StreamExt};
use tokio::sync::{mpsc, Mutex};

#[tokio::main]
async fn main() {
    // Setup an API client:
    let token: Vec<String> = env::args().collect();

    let api = Api::new(&token[1]).unwrap();

    // Dispatcher takes a context which will be passed to each handler
    // we use api client for this, but you can pass any struct.
    let mut dispatcher = Dispatcher::new(api.clone());

    // You also can implement Handler for a struct:
    struct UpdateHandler;

    // note: #[handler] macro expands to something like this
    #[async_trait]
    impl Handler<Api> for UpdateHandler {
        // An object to handle (update, message, inline query, etc...)
        type Input = Update;
        // A result to return
        // You can use Result<T, E>, HandlerResult or ()
        type Output = Result<()>;

        async fn handle(&mut self, context: &Api, input: Self::Input) -> Self::Output {
            let mut response = String::new();

            if let Some(chat_id) = input.get_chat_id() {
                // println!("input: {:?}", input);
                if let UpdateKind::Message(msg) = input.kind {
                    match msg.data {
                        MessageData::Photo { caption, data } => {
                            let getfile = GetFile::new(&data[0].file_id);

                            let y = context.execute(getfile).await;
                            if let Some(file_path) = &y.unwrap().file_path {
                                let mut stream = context.download_file(file_path).await.unwrap();
                                let mut new_file = File::create("foo.png")?;

                                while let Some(chunk) = stream.next().await {
                                    let chunk = chunk.unwrap();
                                    // write chunk to something...

                                    // let mut writer = BufWriter::new(new_file);

                                    new_file.write(&chunk);

                                    response = read_image("./foo.png");
                                }
                            }
                            // println!("{:#?}", y);
                            // println!("{:?}", context);
                        }

                        //Stickers coming soon!
                        MessageData::Sticker(x) => {
                            let file_id = &x.file_id;

                            if let Ok(file_data) = context.execute(GetFile::new(file_id)).await {
                                if let Some(file_path) = file_data.file_path {
                                    if let Ok(mut stream) = context.download_file(file_path).await {
                                        let mut new_file = File::create("foo.webp");
                                        if let Ok(mut file) = new_file {
                                            while let Some(chunk) = stream.next().await {
                                                let chunk = chunk.unwrap();
                                                file.write(&chunk);

                                                response = read_image("./foo.webp");
                                            }
                                        }
                                    }
                                }
                            }

                            // println!("{:#?}", y);
                            // println!("{:?}", x);
                        }
                        (_) => (),
                    }
                }

                context.execute(SendMessage::new(chat_id, response)).await;
            }
            Ok(())
        }
    }
    fn read_image(filename: &str) -> String {
        let mut api = leptess::tesseract::TessApi::new(Some("./tessdata"), "fin").unwrap();
        let mut pix = leptess::leptonica::pix_read(Path::new(filename)).unwrap();
        api.set_image(&pix);

        let text = api.get_utf8_text();
        text.unwrap()
    }

    dispatcher.add_handler(UpdateHandler);

    // in order to catch errors occurred in handlers you can set an error hander:

    // log error and go to the next handler
    dispatcher.set_error_handler(LoggingErrorHandler::new(ErrorPolicy::Continue));
    // by default dispatcher logs error and stops update propagation (next handler will not run)

    // now you can start your bot:

    // using long polling
    LongPoll::new(api, dispatcher).run().await;
}
