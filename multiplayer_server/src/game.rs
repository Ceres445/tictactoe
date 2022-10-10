use tokio::time::delay_for;

// use crate::events::{Client, ClientEvent, PlayerData, ServerGameState, Session};

use nanoid::nanoid;
use serde_json::from_str;
use std::{collections::HashMap, time::Duration};
use warp::{filters::BoxedFilter, Reply};
use websocket_server::{
    message_client as server_msg_client, server,
    sessions::{Client, Clients, Session as OtherSession, Sessions as OtherSessions},
    SafeClients, SafeSessions as OtherSafeSessions, ServerConfig,
};

use crate::lib::{ClientEvent, PlayerData, Players, ServerApp, ServerEvent, ServerGameState};

// pub type Sessions = HashMap<String, Session>;
// pub type SafeSessions = Arc<RwLock<Sessions>>;

// pub type SafeClients = Arc<RwLock<Clients>>;
// pub type Clients = HashMap<String, Client>;

/// This will take a lot of bandwidth if the rate is too high
///

pub fn message_client(client: &Client, message: &ServerEvent) {
    log::debug!(
        "sent message to client: {} \n {:?}",
        client.id,
        serde_json::to_string(message).unwrap()
    );
    server_msg_client(client, message);
}
type SafeSessions = OtherSafeSessions<ServerGameState>;
type Sessions = OtherSessions<ServerGameState>;
type Session = OtherSession<ServerGameState>;

pub async fn tick_handler(_clients: SafeClients, sessions: SafeSessions) {
    loop {
        for session in sessions.write().await.values_mut() {
            let _game_data = &mut session.data;

            // game_data.game.clear_warning_message();
            // todo!();

            delay_for(Duration::from_millis(2000)).await;
        }
    }
}

pub async fn handle_event(client_id: String, event: String, clients: SafeClients, sessions: SafeSessions) {
    //======================================================
    // Deserialize into Session Event object
    //======================================================
    let client_event = match from_str::<ClientEvent>(&event) {
        Ok(obj) => obj,
        Err(_) => {
            if let Some(client) = clients.write().await.get(&client_id) {
                message_client(
                    client,
                    &ServerEvent::Error(format!("Invalid Client Event: {}", &event).to_string()),
                );
            } else {
                log::error!("failed to get client from clients");
            };
            return log::error!("Failed to parse client event");
        }
    };

    match client_event {
        ClientEvent::ListSessions => {
            let sessions = sessions.read().await;
            let mut session_ids = vec![];
            for session in sessions.values() {
                session_ids.push(session.id.clone());
            }
            if let Some(client) = clients.write().await.get(&client_id) {
                message_client(client, &ServerEvent::ListSessions(session_ids));
            } else {
                log::error!("failed to get client from clients");
            }
        }
        ClientEvent::CreateSession => {
            log::info!("request from <{}> to create new session", client_id);

            let session_id = {
                let sessions = &mut sessions.write().await;
                match create_session(None, sessions) {
                    Ok(id) => id,
                    Err(_) => return log::error!("failed to create session.."),
                }
            };

            if let Some(client) = clients.write().await.get_mut(&client_id) {
                if let Some(session) = sessions.write().await.get_mut(&session_id) {
                    start_game(client, session).await.unwrap();
                } else {
                    log::error!("failed to get session {} from sessions", session_id);
                }
            } else {
                log::error!("failed to get client {} from clients", client_id);
            }
        }
        ClientEvent::JoinSession(session_id) => {
            log::info!("request from <{}> to join session {}", client_id, session_id);

            // If the Session does not exists then we will create it first
            log::debug!(
                "checking if session {} exists in {:?}",
                session_id,
                sessions.read().await.keys().collect::<Vec<_>>()
            );
            if sessions.read().await.get(&session_id).is_none() {
                let mut_sessions = &mut sessions.write().await;
                create_session(Some(&session_id), mut_sessions).expect("unable to create a session with a given id.");
            }

            let result = if let Some(client) = clients.write().await.get_mut(&client_id) {
                if let Some(session) = sessions.write().await.get_mut(&session_id) {
                    start_game(client, session).await
                } else {
                    log::error!("failed to get session {} from sessions", session_id);
                    StartResult::Ok
                }
            } else {
                log::error!("failed to get client {} from clients", client_id);
                StartResult::Ok
            };

            if let StartResult::Send(client, event) = result {
                if let Some(client) = clients.write().await.get_mut(&client) {
                    message_client(client, &event);
                } else {
                    log::error!("failed to get client {} from clients", client_id);
                }
            };
        }

        // if let Some(client) = clients.write().await.get_mut(&client_id) {
        //     log::warn!("sending map data..");
        //     message_client(
        //         client,
        //         &ServerEvent::MapUpdate(MAPS.get("first").unwrap().tile_data.clone()),
        //     );
        // }
        ClientEvent::LeaveSession => {
            if let Some(client) = clients.write().await.get_mut(&client_id) {
                let sessions = &mut sessions.write().await;
                remove_client_from_current_session(client, sessions);
            }
        }
        ClientEvent::GameEvent(action) => {
            if let Some(client) = clients.write().await.get_mut(&client_id) {
                // let sessions = &mut sessions.write().await;
                let session_id = {
                    let clients = clients.read().await;
                    pull_client_session_id(&client_id, &clients).unwrap()
                };
                if let Some(session) = sessions.write().await.get_mut(&session_id) {
                    let data = &mut session.data;
                    let game = &mut data.game.as_ref().unwrap().clone();
                    if let Players::Full(_, _, current_player) = data.players.clone() {
                        if client.id == current_player {
                            match game.update(action) {
                                Ok(_) => {
                                    for client_id in session.client_status.keys() {
                                        if let Some(client) = clients.write().await.get(&client_id.clone()) {
                                            message_client(client, &ServerEvent::GameUpdate(game.clone()));
                                        }
                                    }
                                    data.game = Some(game.clone());
                                }
                                Err(e) => message_client(client, &ServerEvent::Error(e)),
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Creates a new empty Session
///
/// Takes a predefined ID to generate, or uses a randomly generated String
pub fn create_session(session_id: Option<&str>, sessions: &mut Sessions) -> Result<String, ()> {
    log::info!("creating session..");
    let session = &mut Session {
        client_status: HashMap::new(),
        id: match session_id {
            Some(id) => String::from(id),
            None => generate_session_id(SESSION_ID_LENGTH),
        },
        data: ServerGameState::default(),
    };

    log::info!("writing new session {} to global sessions", session.id);
    // add a new session into the server
    sessions.insert(session.id.clone(), session.clone());

    log::info!("finished creating session {}", session.id);
    log::info!("sessions live: {}", sessions.len());

    Ok(session.id.clone())
}

/// Removes a client from the session that they currently exist under
fn remove_client_from_current_session(client: &mut Client, sessions: &mut Sessions) {
    log::info!("attempting to remove client {} from their current session", client.id);

    let session_id = match &client.session_id {
        Some(id) => String::from(id),
        None => return log::warn!("client {} was not in a session", client.id),
    };

    match sessions.get_mut(&session_id) {
        Some(session) => {
            // remove the client from the session
            session.remove_client(&client.id);

            log::info!("removed client {} from session {}", client.id, session_id);
            // revoke the client's reference to the current Session ID
            client.session_id = None;
            // clean up the session from the map if it is empty
            if session.get_clients_with_active_status(true).is_empty() {
                cleanup_session(&session_id, sessions);
            }
        }
        None => log::error!("failed to find session {} to remove client {}", session_id, client.id),
    }
}

/// Takes a mutable session reference in order to add a client to a given session
///
/// Takes a Read lock for Clients
///
enum StartResult {
    Send(String, ServerEvent),
    Ok,
}

impl StartResult {
    fn unwrap(self) {
        match self {
            StartResult::Send(_, event) => return log::info!("unwrapping event: {:?}", event),
            StartResult::Ok => (),
        }
    }
}

async fn start_game(client: &mut Client, session: &mut Session) -> StartResult {
    // add client to session
    log::info!("attempting to add client {} to session {}", client.id, session.id);
    match session.data.clone().players {
        Players::Full(..) => message_client(client, &ServerEvent::Error("Session is full".to_string())),
        Players::Partial(player) => {
            session.insert_client(&client.id, true);
            log::info!("attaching session {} to client <{}>", session.id, client.id);
            client.session_id = Some(session.id.clone());
            log::info!("client <{}> joined session: <{}>", client.id, session.id);
            log::info!(
                "Session {} has {} clients, starting game",
                session.id,
                session.get_clients_with_active_status(true).len()
            );
            session.data.game = Some(ServerApp::default());
            session.data.players = Players::Full(
                player.clone(),
                PlayerData::new(&client.id),
                player.id.clone(), // TODO: randomize this
            );
            let data = session.data.clone();
            message_client(client, &ServerEvent::GameStart(data.clone()));
            return StartResult::Send(player.id.clone(), ServerEvent::GameStart(data.clone()));
        }
        Players::Empty => {
            session.insert_client(&client.id, true);
            log::info!("attaching session {} to client <{}>", session.id, client.id);
            client.session_id = Some(session.id.clone());
            session.data.players = Players::Partial(PlayerData::new(&client.id));
            log::info!("client <{}> joined session: <{}>", client.id, session.id);
            log::info!(
                "Session {} has {} clients, sending queue",
                session.id,
                session.get_clients_with_active_status(true).len()
            );
            message_client(client, &ServerEvent::Queue(session.id.clone()))
        }
    };

    StartResult::Ok
    // add client to gamedata
    // session
    //     .data
    //     .players
    //     .insert(client.id.clone(), PlayerData::new(&client.id));

    // if session.data.players.len() == 2 {
    //     session.data.game = Some(Game::new(tictactoe_library::update::Opponent::Human));
    //     session.data.current_player = Some(session.data.players.into_keys().next().unwrap());
    //     Ok(true)
    // } else {
    //     Ok(false)
    // }
}

/// The Chosen Length of a Session ID
pub const SESSION_ID_LENGTH: usize = 5;

/// Generates a String of given length using characters that are valid for Session IDs
///
/// This should effectively resolve to Session uniqueness when the length is
/// greater than a value like 4 for a plausable number of concurrent sessions
fn generate_session_id(length: usize) -> String {
    nanoid!(
        length,
        &[
            'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U',
            'V', 'W', 'X', 'Y', 'Z',
        ]
    )
}

/// pull the session id off of a client
fn pull_client_session_id(client_id: &str, clients: &Clients) -> Option<String> {
    match clients.get(client_id) {
        Some(client) => client.session_id.clone(),
        None => None,
    }
}

pub fn get_server() -> BoxedFilter<(impl Reply,)> {
    let config = ServerConfig {
        tick_handler: Some(tick_handler),
        event_handler: handle_event,
    };
    server(config)
}

/// Removes a session from the map if it is is_empty
pub fn cleanup_session(session_id: &str, sessions: &mut Sessions) {
    // remove session
    sessions.remove(session_id);
    // log status
    log::info!("removed empty session");
    log::info!("sessions live: {}", sessions.len());
}

#[cfg(test)]
mod tests {
    // use std::sync::Arc;

    use serde_json::to_string;
    // use tokio::sync::RwLock;
    use super::*;
    use test_log::test;
    use warp::{test::request, ws::Message};

    #[test]
    fn test_generate_session_id() {
        let id = generate_session_id(5);
        assert_eq!(id.len(), 5);
    }

    #[tokio::test]
    async fn test_client_handler() {
        // let clients: SafeClients = Arc::new(RwLock::new(HashMap::new()));
        // let sessions: SafeSessions = Arc::new(RwLock::new(HashMap::new()));
        let server = get_server();
        let request = request()
            .method("GET")
            .path("/api/health")
            .matches(&server.to_owned())
            .await;
        assert_eq!(request, true);
    }

    async fn test_ws() {
        let server = get_server();
        let mut client = warp::test::ws()
            .path("/api/ws/test")
            .handshake(server)
            .await
            .expect("handshake");
        client
            .send(Message::text(
                to_string(&ClientEvent::JoinSession("Hello".to_string())).unwrap(),
            ))
            .await;

        assert_eq!(
            serde_json::from_str::<ServerEvent>(client.recv().await.unwrap().to_str().unwrap()).expect("deserialize"),
            ServerEvent::Queue("Hello".to_string())
        );
        client.send_text(to_string(&ClientEvent::ListSessions).unwrap()).await;
        assert_eq!(
            serde_json::from_str::<ServerEvent>(client.recv().await.unwrap().to_str().unwrap()).expect("deserialize"),
            ServerEvent::ListSessions(vec!["Hello".to_string()])
        );
    }

    // #[test(tokio::test)]
    // async fn test_game_logic() {
    //     let mut client = warp::test::ws()
    //         .path("/api/ws/test")
    //         .handshake(get_server())
    //         .await
    //         .expect("handshake");

    //     client
    //         .send(Message::text(
    //             to_string(&ClientEvent::JoinSession("Hello".to_string())).unwrap(),
    //         ))
    //         .await;
    //     assert_eq!(
    //         serde_json::from_str::<ServerEvent>(client.recv().await.unwrap().to_str().unwrap()).expect("deserialize"),
    //         ServerEvent::Queue("Hello".to_string())
    //     );

    //     let mut client2 = warp::test::ws()
    //         .path("/api/ws/test")
    //         .handshake(get_server())
    //         .await
    //         .expect("handshake");
    //     client.send_text(to_string(&ClientEvent::ListSessions).unwrap()).await;
    //     assert_eq!(
    //         serde_json::from_str::<ServerEvent>(client.recv().await.unwrap().to_str().unwrap()).expect("deserialize"),
    //         ServerEvent::ListSessions(vec!["Hello".to_string()])
    //     );
    //     client2
    //         .send_text(to_string(&ClientEvent::JoinSession("Hello".to_string())).unwrap())
    //         .await;
    //     let game_state = ServerGameState {
    //         players: Players::Full(
    //             PlayerData::new(&"test".to_string()),
    //             PlayerData::new(&"another".to_string()),
    //             "test".to_string(),
    //         ),
    //         game: Some(ServerApp::default()),
    //     };
    //     assert_eq!(
    //         serde_json::from_str::<ServerEvent>(client2.recv().await.unwrap().to_str().unwrap()).expect("deserialize"),
    //         ServerEvent::GameStart(game_state)
    //     );
    // }
}
