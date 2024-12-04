use tungstenite::connect;
use url::Url;

fn main() {
    let jwt = "jwt";
    let mut url = Url::parse("ws://localhost:9003/api/v1/ws").unwrap();
    url.query_pairs_mut()
        .append_pair("Authorization", &format!("Bearer {}", jwt));
    println!("Using url... {:?}", url.as_str());

    let (mut _socket, response) = connect(url.as_str()).expect("Can't connect");

    println!("Connected to the server");
    println!("Response HTTP code: {}", response.status());
    println!("Response contains the following headers:");
    for (ref header, _value) in response.headers() {
        println!("* {}", header);
    }

    // let sub_msg = ClientMessage::Ping;
    // socket
    //     .write_message(Message::Text(serde_json::to_string(&sub_msg).unwrap()))
    //     .unwrap();
    // loop {
    //     let msg = socket.read_message().expect("Error reading message");
    //     println!("Received: {}", msg);
    // }
    // socket.close(None);
}
