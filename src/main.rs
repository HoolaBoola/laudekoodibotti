use std::env;
use tokio::net::{TcpListener, TcpStream};
use tokio::stream::{Stream, StreamExt};
use tokio::sync::{mpsc, Mutex};

#[tokio::main]
async fn main() {
    // Setup an API client:
    use carapax::{Api, Config};
    let token: Vec<String> = env::args().collect();

    let api = Api::new(&token[1]).unwrap();

    // And dispatcher:
    use carapax::Dispatcher;

    // Dispatcher takes a context which will be passed to each handler
    // we use api client for this, but you can pass any struct.
    let mut dispatcher = Dispatcher::new(api.clone());

    // Let's add a command handler
    use carapax::{handler, types::Command};

    #[handler(command = "/start")]
    async fn command_handler(_context: &Api, _command: Command) {
        // handler takes a reference to context passed to dispathcer
    }

    dispatcher.add_handler(command_handler);

    // A message handler:
    use carapax::{types::Message, HandlerResult};

    #[handler]
    async fn message_handler(_context: &Api, _message: Message) -> HandlerResult {
        // handle message here...

        // say that next handler will run
        HandlerResult::Continue
        // but you can prevent next handler by using HandlerResult::Stop
        // or return an error using `HandlerResult::Error`: Err(err).into()
        // In case of error, next handler will not run by default. See below how to change this behavior.
    }

    dispatcher.add_handler(message_handler);

    // You also can implement Handler for a struct:
    struct UpdateHandler;

    use carapax::methods::SendMessage;
    use carapax::types::Update;
    use carapax::{async_trait, ExecuteError, Handler};

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
                context.execute(SendMessage::new(chat_id, "Hello!")).await?;
            }
            Ok(())
        }
    }

    dispatcher.add_handler(UpdateHandler);

    // in order to catch errors occurred in handlers you can set an error hander:

    use carapax::{ErrorHandler, ErrorPolicy, HandlerError, LoggingErrorHandler};

    // log error and go to the next handler
    dispatcher.set_error_handler(LoggingErrorHandler::new(ErrorPolicy::Continue));
    // by default dispatcher logs error and stops update propagation (next handler will not run)

    // or you can implement your own error handler:

    struct MyErrorHandler;

    #[async_trait]
    impl ErrorHandler for MyErrorHandler {
        async fn handle(&mut self, err: HandlerError) -> ErrorPolicy {
            ErrorPolicy::Continue
        }
    }

    dispatcher.set_error_handler(MyErrorHandler);

    // now you can start your bot:

    // using long polling
    use carapax::longpoll::LongPoll;

    LongPoll::new(api, dispatcher).run().await;

    // or webhook
    // use carapax::webhook::run_server;
    // run_server(([127, 0, 0, 1], 8080), "/path", dispatcher).await.unwrap();
}
