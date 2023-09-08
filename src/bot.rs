use crate::handlers::command::*;
use crate::handlers::system::*;
use crate::settings::Settings;
use std::convert::Infallible;
use teloxide::dispatching::MessageFilterExt;
use teloxide::dispatching::{Dispatcher, UpdateFilterExt};
use teloxide::dptree;
use teloxide::prelude::LoggingErrorHandler;
use teloxide::types::{Message, Update};
use teloxide::{update_listeners::UpdateListener, Bot};

pub async fn start_bot(
    bot: Bot,
    listener: impl UpdateListener<Err = Infallible>,
    settings: Settings,
) {
    let handler = dptree::entry()
        // .inspect(|u: Update| println!("{:#?}", u))
        .branch(
            Update::filter_message()
                .branch(
                    teloxide::filter_command::<GeneralCommand, _>()
                        .endpoint(GeneralCommand::parse_commands),
                )
                .branch(
                    teloxide::filter_command::<AdminCommand, _>()
                        .filter(|msg: Message| msg.chat.is_private())
                        .endpoint(AdminCommand::parse_commands),
                )
                .branch(Message::filter_new_chat_members().endpoint(handle_new_member))
                .branch(Message::filter_left_chat_member().endpoint(handle_left_member)),
        );

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![settings])
        .enable_ctrlc_handler()
        .build()
        .dispatch_with_listener(listener, LoggingErrorHandler::new())
        .await;
}