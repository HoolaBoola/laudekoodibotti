use carapax::longpoll::LongPoll;
use carapax::methods::SendMessage;
use carapax::types::Update;
use carapax::types::UpdateKind;
use carapax::Dispatcher;
use carapax::{async_trait, ExecuteError, Handler};
use carapax::{handler, types::Command};
use carapax::{types::Message, HandlerResult};
use carapax::{Api, Config};
use carapax::{ErrorHandler, ErrorPolicy, HandlerError, LoggingErrorHandler};
use std::env;
use tokio::net::{TcpListener, TcpStream};
use tokio::stream::{Stream, StreamExt};
use tokio::sync::{mpsc, Mutex};
use carapax::types::MessageData;
use carapax::methods::GetFile;

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
        type Output = Result<(), ExecuteError>;

        async fn handle(&mut self, context: &Api, input: Self::Input) -> Self::Output {
            if let Some(chat_id) = input.get_chat_id() {
                // println!("input: {:?}", input);
                if let UpdateKind::Message(msg) = input.kind {
                    match msg.data {
                        MessageData::Photo{caption, data} => {
                            let getfile = GetFile::new(&data[0].file_id);
                            
                            context.execute(getfile);
                            // println!("{:?}", context);

                        },
                        MessageData::Sticker(x) => {
                            let getfile = GetFile::new(&x.file_id);
                            context.execute(getfile);
                            // println!("{:?}", x);
                        },
                        (_) => (),
                    }
                }
                // match input.kind {
                //     UpdateKind::Message(Message) => {
                //         if let
                //         println!("{:?}", Message.data)
                //     },
                //     (_) => (),
                // }
                context.execute(SendMessage::new(chat_id, "Hello!")).await?;
            }
            Ok(())
        }
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
