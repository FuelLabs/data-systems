use fuel_streams_ws::server::{
    http::models::{LoginRequest, LoginResponse},
    ws::models::{
        ClientMessage,
        ServerMessage,
        SubscriptionPayload,
        SubscriptionType,
    },
};
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use tungstenite::{
    handshake::client::{generate_key, Request},
    Message,
};
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // get jwt token
    let jwt_url = Url::parse("http://localhost:9003/api/v1/jwt").unwrap();
    let client = reqwest::Client::new();
    let json_body = serde_json::to_string(&LoginRequest {
        username: "client".to_string(),
        password: "client".to_string(),
    })?;
    let response = client
        .get(jwt_url)
        .header(ACCEPT, "application/json")
        .header(CONTENT_TYPE, "application/json")
        .body(json_body)
        .send()
        .await?;
    let jwt = if response.status().is_success() {
        let json_body = response.json::<LoginResponse>().await?;
        println!("Jwt endpoint response JSON: {:?}", json_body);
        json_body.jwt_token
    } else {
        panic!("Failed to fetch jwt data: {}", response.status());
    };

    // open websocket connection
    let url = Url::parse("ws://localhost:9003/api/v1/ws").unwrap();
    println!("Using websocket url: {:?}", url.as_str());

    // url.query_pairs_mut()
    //     .append_pair("Authorization", &urlencoding::encode(&format!("Bearer {}", jwt)));
    // let (mut socket, response) = connect(url.as_str()).expect("Can't connect to ws");

    // Create the WebSocket request with the Authorization header
    let host = url.host_str().expect("Invalid host");
    let request = Request::builder()
        .uri(url.as_str())
        .header("Authorization", format!("Bearer {}", jwt))
        .header("Host", host)
        .header("Upgrade", "websocket")
        .header("Connection", "Upgrade")
        .header("Sec-WebSocket-Key", generate_key())
        .header("Sec-WebSocket-Version", "13")
        .body(())
        .expect("Failed to build request");

    let (mut socket, response) =
        match tungstenite::client::connect_with_config(request, None, 5) {
            Ok((socket, response)) => (socket, response),
            Err(err) => {
                eprintln!(
                    "Failed to connect to the server: {:?}",
                    err.to_string()
                );
                panic!("Failed to connect to the server");
            }
        };

    for (header, value) in response.headers() {
        println!("* {}: {:?}", header, value);
    }

    println!("Connected to the server");
    println!("Response HTTP code: {}", response.status());
    println!("Response contains the following headers:");
    for (ref header, _value) in response.headers() {
        println!("* {}", header);
    }

    let stream_topic_wildcard = "blocks.*.*".to_owned();
    let msg = ClientMessage::Subscribe(SubscriptionPayload {
        topic: SubscriptionType::Stream(stream_topic_wildcard.clone()),
    });
    socket
        .send(Message::Text(serde_json::to_string(&msg).unwrap()))
        .unwrap();

    socket
        .send(Message::Binary(serde_json::to_vec(&msg).unwrap()))
        .unwrap();

    socket
        .send(Message::Ping(serde_json::to_vec(&msg).unwrap()))
        .unwrap();

    socket
        .send(Message::Pong(serde_json::to_vec(&msg).unwrap()))
        .unwrap();

    let bad_sub = serde_json::json!({
        "subscribe": {
            "topics": {
                "stream": stream_topic_wildcard
            }
        }
    });
    socket
        .send(Message::Binary(serde_json::to_vec(&bad_sub).unwrap()))
        .unwrap();

    let jh = tokio::spawn(async move {
        loop {
            let msg = socket.read();
            println!("Received: {:?}", msg);
            match msg {
                Ok(msg) => match msg {
                    Message::Text(text) => {
                        println!("Received text: {:?}", text);
                    }
                    Message::Binary(bin) => {
                        println!("Received binary: {:?}", bin);
                        let decoded =
                            serde_json::from_slice::<ServerMessage>(&bin)
                                .unwrap();
                        println!("Received server message: {:?}", decoded);
                    }
                    Message::Ping(ping) => {
                        println!("Received ping: {:?}", ping);
                    }
                    Message::Pong(pong) => {
                        println!("Received pong: {:?}", pong);
                    }
                    Message::Close(close) => {
                        println!("Received close: {:?}", close);
                        break;
                    }
                    _ => {
                        println!("Received unknown message type");
                    }
                },
                Err(e) => {
                    println!("Error reading message: {:?}", e);
                    break;
                }
            }
        }
    });

    jh.await?;

    Ok(())
}
