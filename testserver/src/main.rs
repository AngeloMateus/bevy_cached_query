use std::collections::HashMap;
use tiny_http::{Response, Server};

fn main() {
    let server = Server::http("127.0.0.1:8080").unwrap();
    let responses = HashMap::from([
        ("/", "{\"msg\": \"\"}"),
        ("/extractor", "{\"msg\": \"hello world\"}"),
        ("/force_next_refetch", "{\"msg\": \"Should be consumed once\"}"),
        ("/same_url", "{\"msg\": \"success\"}"),
        ("/same_url?second_request=is_discarded", "{\"msg\": \"fail\"}"),
        ("/is_stale", "{\"msg\": \"Should not be consumed\"}"),
        ("/refetch", "{\"msg\": \"Should refetch\"}"),
        ("/seq1", "{\"msg\": \"1\"}"),
        ("/seq2", "{\"msg\": \"2\"}"),
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
