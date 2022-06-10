mod routes;
mod server;

use server::HttpServer;
use std::{thread, time};

fn main() {
    let mut server = HttpServer::new();
    server.get("/my-path", &|req| {
        return "ABC".to_string();
    });
    server.get("/", &routes::login);
    server.get("/favicon.ico", &routes::fourohfor);
    server.get("/home", &routes::home);
    server.get("/sleep", &|req| {
        thread::sleep(time::Duration::from_secs(5));
        return "".to_string();
    });
    server.post("/home", &|req| {
        return "POST HOME HANDLED".to_string();
    });
    server.listen();
}
