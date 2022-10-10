use futures_util::{sink::SinkExt, StreamExt};
use multiplayer_server::{ClientEvent, ServerEvent};
use websocket_lite::{AsyncNetworkStream, ClientBuilder, Message, Opcode, Result as AsyncResult};
type AsyncClient = websocket_lite::AsyncClient<Box<dyn AsyncNetworkStream + Sync + Send + Unpin + 'static>>;

pub struct Client {
    ws: Option<AsyncClient>,
    pub name: String,
}

impl Client {
    pub fn new(name: String) -> Self {
        Self { ws: None, name }
    }

    pub async fn get_ws(&mut self) -> Result<&mut AsyncClient, String> {
        let url = url::Url::parse(&"ws://localhost:8000/api/ws/{}".replace("{}", self.name.as_str())).unwrap();
        if self.ws.is_none() {
            let ws = ClientBuilder::from_url(url)
                .async_connect()
                .await
                .expect("Could not connect to server");
            self.ws = Some(ws);
        }
        Ok(self.ws.as_mut().unwrap())
    }

    // pub async fn connect(&mut self) -> Result<()> {
    //     let client = ClientBuilder::from_url(url);
    //     self.ws = Some(client.async_connect().await?);

    //     println!("Connected to server");
    // loop {
    //     let msg: Option<Result<Message>> = self.ws.next().await;

    //     let msg = if let Some(msg) = msg {
    //         msg
    //     } else {
    //         break;
    //     };

    //     let msg = if let Ok(msg) = msg {
    //         msg
    //     } else {
    //         let _ = self.ws.send(Message::close(None)).await;
    //         break;
    //     };

    //     match msg.opcode() {
    //         Opcode::Text => {
    //             println!("{}", msg.as_text().unwrap());
    //             self.ws.send(msg).await?
    //         }
    //         Opcode::Binary => self.ws.send(msg).await?,
    //         Opcode::Ping => self.ws.send(Message::pong(msg.into_data())).await?,
    //         Opcode::Close => {
    //             let _ = self.ws.send(Message::close(None)).await;
    //             break;
    //         }
    //         Opcode::Pong => {}
    //     }
    // Ok(())
    // }

    pub async fn send(&mut self, msg: ClientEvent) -> Result<ServerEvent, String> {
        let server_msg = Message::text(serde_json::to_string(&msg).expect("serialize"));
        let ws = self.get_ws().await.unwrap();
        log::info!("Sent message: {:?}", &serde_json::to_string(&msg).unwrap());
        ws.send(server_msg).await.expect("send");
        self.recv().await
    }

    pub async fn recv(&mut self) -> Result<ServerEvent, String> {
        let ws = self.get_ws().await.unwrap();
        loop {
            let msg: Option<AsyncResult<Message>> = ws.next().await;
            if let Some(Ok(msg)) = msg {
                match msg.opcode() {
                    Opcode::Text => {
                        let msg = serde_json::from_str::<ServerEvent>(msg.as_text().unwrap()).expect("deserialize");
                        log::info!("Received message: {:?}", msg);
                        return Ok(msg);
                    }
                    _ => log::debug!("Wrong opcode: {:?}", msg.opcode()),
                }
            } else {
                return Err("Did not receive reply message".to_string());
            }
        }
    }
}

// tests
#[cfg(test)]
mod tests {

    use super::*;
    use test_log::test;
    use tokio::time::{sleep, Duration};

    #[test(tokio::test(flavor = "multi_thread", worker_threads = 2))]
    async fn test_client() {
        let _ = env_logger::builder().is_test(true).try_init();
        let task1 = tokio::spawn(async move {
            let mut client = Client::new("test".to_string());
            assert_eq!(
                ServerEvent::Queue("New".to_string()),
                client.send(ClientEvent::JoinSession("New".to_string())).await.unwrap()
            );
            log::info!(" Client 1 joined session");
            assert!(matches!(client.recv().await.unwrap(), ServerEvent::GameStart(..)));
            log::info!(" Client 1 received server start");
        });

        let task2 = tokio::spawn(async move {
            sleep(Duration::from_millis(1000)).await;
            let mut client = Client::new("another".to_string());
            assert!(matches!(
                client.send(ClientEvent::JoinSession("New".to_string())).await.unwrap(),
                ServerEvent::GameStart(..)
            ));
            log::info!(" Client 2 joined session");
            // log::info!(" Client 2 received server start");
        });
        let (a, b) = futures::join!(task1, task2);
        a.unwrap();
        b.unwrap();
    }
}
