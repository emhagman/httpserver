use std::fs;

use crate::server::HttpRequest;

pub fn login(req: &HttpRequest) -> String {
    if req.method.expect("") == "GET" {
        let contents = fs::read_to_string("hello.html").expect("failed to read html file");
        return contents;
    } else {
        let contents = fs::read_to_string("hello2.html").expect("failed to read html file");
        return contents;
    }
}

pub fn home(req: &HttpRequest) -> String {
    if req.method.expect("") == "GET" {
        let contents = fs::read_to_string("hello.html").expect("failed to read html file");
        return contents;
    } else {
        let contents = fs::read_to_string("hello2.html").expect("failed to read html file");
        return contents;
    }
}

pub fn fourohfor(req: &HttpRequest) -> String {
    return "NOT_FOUND".to_string();
}
