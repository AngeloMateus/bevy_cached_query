use std::collections::HashMap;
use tiny_http::{Response, Server};

fn main() {
    let server = Server::http("127.0.0.1:8080").unwrap();
    let responses = HashMap::from([
        ("/extractor", "{\"msg\": \"hello world\"}"),
        ("/same_url", "{\"msg\": \"success\"}"),
        ("/same_url?second_request=is_discarded", "{\"msg\": \"fail\"}"),
    ]);
    loop {
        let request = server.recv();

        if let Ok(request) = request {
            if let Some(response) = responses.get(&request.url()) {
                let response = Response::from_string(response.to_string());
                request.respond(response).expect("Responded");
            }
        }
    }
}
