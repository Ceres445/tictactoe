use std::{collections::HashMap, sync::Arc};

use crate::{
    events::Client,
    game::{SafeClients, SafeSessions},
    handler::{health_handler, ws_handler},
};
use tokio::sync::{Mutex, RwLock};
use warp::{filters::BoxedFilter, Filter, Rejection, Reply};

type Result<T> = std::result::Result<T, Rejection>;
type Clients = Arc<Mutex<HashMap<String, Client>>>;

pub fn server() -> BoxedFilter<(impl Reply,)> {
    let clients: SafeClients = Arc::new(RwLock::new(HashMap::new()));
    let sessions: SafeSessions = Arc::new(RwLock::new(HashMap::new()));

    let health_route = warp::path!("health").and_then(health_handler);

    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(warp::path::param())
        .and(warp::any().map(move || clients.clone()))
        .and(warp::any().map(move || sessions.clone))
        .and(warp::any().map(move || crate::game::handle_event))
        .and_then(ws_handler);

    health_route.or(ws_route).with(warp::cors().allow_any_origin()).boxed()
}
