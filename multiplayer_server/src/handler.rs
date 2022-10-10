use futures::Future;
use log::info;
use tokio::sync::mpsc;
use urlencoding::decode;
use warp::{hyper::StatusCode, reply::with_status, ws::WebSocket, Reply};

use crate::{
    events::Client,
    game::{cleanup_session, SafeClients, SafeSessions, Sessions},
};

#[derive(Debug)]
pub struct IDAlreadyTaken;
impl warp::reject::Reject for IDAlreadyTaken {}

pub async fn ws_handler<EventHandler>(
    ws: warp::ws::Ws,
    id: String,
    clients: SafeClients,
    sessions: SafeSessions,
    event_handler: EventHandler,
) -> Result<impl Reply> {
    let client_exists = clients.read().await.get(&id).is_none();
    match client_exists {
        false => {
            log::warn!("duplicate connection request for id: {}", id);
            Err(warp::reject::custom(IDAlreadyTaken))
        }
        true => Ok(ws.on_upgrade(move |socket| {
            log::info!("incoming request for id: {}", id);
            client_connection(socket, id, clients, sessions, event_handler)
        })),
    }
}

/// Health Check Endpoint used to verify the service is live
pub async fn health_handler() -> Result<impl Reply> {
    info!("HEALTH_CHECK ✓");
    Ok(with_status("health check ✓", StatusCode::OK))
}

pub async fn client_connection<T, Fut, EventHandler>(
    ws: WebSocket,
    connection_id: String,
    clients: SafeClients,
    sessions: SafeSessions,
    event_handler: EventHandler,
) where
    Fut: Future<Output = ()>,
    EventHandler: Fn(String, String, SafeClients, SafeSessions) -> Fut,
{
    // Decode the strings coming in over URL parameters so we dont get things like '%20'
    // for spaces in our clients map
    let id = decode(&connection_id).expect("UTF-8").to_string();
    //======================================================
    // Splits the WebSocket into a Sink + Stream:
    // Sink - Pools the messages to get send to the client
    // Stream - receiver of messages from the client
    //======================================================
    let (client_ws_sender, mut client_ws_rcv) = ws.split();
    //======================================================
    // Gets an Unbounced Channel that can transport messages
    // between asynchronous tasks:
    // Sender - front end of the channel
    // Receiver - recieves the sender messages
    //======================================================
    let (client_sender, client_rcv) = mpsc::unbounded_channel();
    //======================================================
    // Spawn a thread to forward messages
    // from our channel into our WebSocket Sink
    // between asynchronous tasks using the same Client object
    //======================================================
    tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| {
        if let Err(e) = result {
            log::error!("failed to send websocket msg: {}", e);
        }
    }));
    //======================================================
    // From now on we can use our client_sender.send(val: T)
    // to send messages to a given client websocket
    //======================================================

    //======================================================
    // Create a new Client and insert them into the Map
    //======================================================
    clients.write().await.insert(
        id.clone(),
        Client {
            id: id.clone(),
            sender: Some(client_sender),
            session_id: {
                let sessions = sessions.read().await;
                get_client_session_id(&id, &sessions)
            },
        },
    );

    if let Some(client) = clients.read().await.get(&id) {
        handle_client_connect(client, &sessions).await;
    }
    //======================================================
    // Synchronously wait for messages from the
    // Client Receiver Stream until an error occurs
    //======================================================
    while let Some(result) = client_ws_rcv.next().await {
        // Check that there was no error actually obtaining the Message
        match result {
            Ok(msg) => {
                //======================================================
                // Ensure the Message Parses to String
                //======================================================
                let message = match msg.to_str() {
                    Ok(v) => v.clone(),
                    Err(_) => {
                        log::warn!("websocket message: '{:?}' was not handled", msg);
                        log::warn!("disconnecting client <{}>", id);
                        if let Some(client) = clients.write().await.remove(&id) {
                            handle_client_disconnect(&client, &sessions).await;
                        }
                        return;
                    }
                };
                //======================================================
                // pass the message to the event handler
                //======================================================
                event_handler(id.clone(), message.to_string(), clients.clone(), sessions.clone()).await;
            }
            Err(e) => {
                log::error!("failed to recieve websocket message for id: <{}>, error: {}", id, e,);
            }
        }
    }
    //======================================================
    // Remove the Client from the Map
    // when they are finished using the socket (or error)
    //======================================================
    if let Some(client) = clients.write().await.remove(&id) {
        handle_client_disconnect(&client, &sessions).await;
    }
}

/// If a client exists in a session, then set their status to inactive.
///
/// If setting inactive status would leave no other active member, remove the session
async fn handle_client_disconnect(client: &Client, sessions: &SafeSessions) {
    log::info!("client <{}> disconnected", client.id);
    if let Some(session_id) = &client.session_id {
        let mut session_empty = false;
        // remove the client from the session and check if the session become empty
        if let Some(session) = sessions.write().await.get_mut(session_id) {
            if let Err(msg) = session.set_client_active_status(&client.id, false) {
                log::error!("{}", msg);
            }

            session_empty = session.get_clients_with_active_status(true).is_empty();
        }
        // remove the session if empty
        if session_empty {
            let mut write_sessions = sessions.write().await;
            cleanup_session(session_id, &mut write_sessions);
        }
    }
}

/// If a client exists in a session, then set their status to active
async fn handle_client_connect(client: &Client, sessions: &SafeSessions) {
    log::info!("{} connected", client.id);
    if let Some(session_id) = &client.session_id {
        if let Some(session) = sessions.write().await.get_mut(session_id) {
            if let Err(msg) = session.set_client_active_status(&client.id, true) {
                log::error!("{}", msg);
            }
        }
    }
}

/// Gets the SessionID of a client if it exists
fn get_client_session_id(client_id: &str, sessions: &Sessions) -> Option<String> {
    for session in sessions.values() {
        if session.contains_client(client_id) {
            return Some(session.id.clone());
        }
    }

    None
}
