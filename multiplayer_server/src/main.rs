use std::env;

use game::get_server;

mod lib;
// mod handler;
mod game;
// mod server;

#[tokio::main]
async fn main() {
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| String::from("8000"))
        .parse()
        .expect("PORT must be a number");

    // initialize env_logging backend for logging
    let _ = env_logger::builder().is_test(true).try_init();

    // // Pass handlers for the server into the ServerConfig to get them initialized with the application
    // let server_config = ServerConfig {
    //     tick_handler: Some(game::tick_handler),
    //     event_handler: game::handle_event,
    // };

    warp::serve(get_server()).run(([0, 0, 0, 0], port)).await;
}
