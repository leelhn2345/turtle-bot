use teloxide::{
    update_listeners::{webhooks, UpdateListener},
    Bot,
};
// dddd, MMMM D YYYY, h:mm:ss A

// luxon
// EEEE, MMMM d yyyy, h:mm:ss a
use crate::{routes::setup_bot_router, settings::Settings};

pub async fn setup_axum_webhook(
    settings: &Settings,
    bot: Bot,
) -> impl UpdateListener<Err = std::convert::Infallible> {
    // let options = webhooks::Options::new(address, url);
    let address = format!(
        "{}:{}",
        settings.application.host, settings.application.port
    )
    .parse()
    .expect("unable to parse host and/or port");

    // domain_path is digital ocean specific

    let url = format!("{}/webhook", settings.application.public_url)
        .parse()
        .expect("unable to parse url");

    let options = webhooks::Options::new(address, url).drop_pending_updates();

    let (mut listener, stop_flag, bot_router) = webhooks::axum_to_router(bot, options)
        .await
        .expect("unable to get listener");

    let router = setup_bot_router(bot_router);

    let stop_token = listener.stop_token();

    tokio::spawn(async move {
        axum::Server::bind(&address)
            .serve(router.into_make_service())
            .with_graceful_shutdown(stop_flag)
            .await
            .map_err(|_| stop_token.stop())
            .expect("axum server error")
    });

    listener
}
