use flate2::write::GzEncoder;
use flate2::Compression;
use std::{
    collections::HashMap,
    io::{BufWriter, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

#[derive(Hash, PartialEq, Eq, Debug)]
enum HttpMethod {
    ANY,
    GET,
    POST,
    HEAD,
    DELETE,
    PATCH,
}

#[derive(Hash, PartialEq, Eq, Debug)]
pub struct HttpRoute<'a>(HttpMethod, &'a str);

type RouteRef = &'static (dyn Fn(&HttpRequest) -> String + Sync);

pub struct HttpServer<'a> {
    address: &'a str,
    routes: Arc<Mutex<HashMap<HttpRoute<'static>, RouteRef>>>,
    listener: Option<TcpListener>,
}

impl<'a> HttpServer<'a> {
    pub fn new() -> Self {
        Self {
            address: "0.0.0.0:7878",
            routes: Arc::new(Mutex::new(HashMap::new())),
            listener: None,
        }
    }
    pub fn listen(&mut self) {
        println!("listening on: {}", self.address);
        let listener = TcpListener::bind(self.address).expect("failed to bind");
        for stream in listener.incoming() {
            let stream = stream.expect("failed to accept request");
            let counter = Arc::clone(&self.routes);
            thread::spawn(move || {
                handle_connection(counter, stream);
            });
        }
        self.listener = Some(listener);
    }
    pub fn any(&mut self, path: &'static str, f: RouteRef) {
        let key = HttpRoute(HttpMethod::ANY, &path);
        self.routes.lock().unwrap().insert(key, f);
    }
    pub fn get(&mut self, path: &'static str, f: RouteRef) {
        let key = HttpRoute(HttpMethod::GET, &path);
        self.routes.lock().unwrap().insert(key, f);
    }
    pub fn post(&mut self, path: &'static str, f: RouteRef) {
        let key = HttpRoute(HttpMethod::POST, &path);
        self.routes.lock().unwrap().insert(key, f);
    }
}

#[derive(Debug)]
pub struct HttpRequest<'a> {
    pub url: Option<&'a str>,
    pub method: Option<&'a str>,
    pub http_version: Option<&'a str>,
    pub headers: Option<&'a Vec<HttpHeader>>,
    pub body: Option<String>,
}

#[derive(Debug)]
pub struct HttpHeader {
    key: String,
    value: String,
}

const BUFFER_SIZE: usize = 1024;

fn handle_connection<'a>(routes: Arc<Mutex<HashMap<HttpRoute<'a>, RouteRef>>>, mut stream: TcpStream) {
    println!("connection established");
    let mut data = Vec::new();
    let mut buf = [0; BUFFER_SIZE];
    loop {
        let bytes_read = stream.read(&mut buf).expect("failed to read the request");
        if bytes_read == 0 {
            break;
        }
        data.extend_from_slice(&buf[..bytes_read]);
        if bytes_read < BUFFER_SIZE {
            break;
        }
    }

    let request = String::from_utf8_lossy(&data);
    let mut request_object = HttpRequest {
        url: None,
        method: None,
        http_version: None,
        headers: None,
        body: None,
    };

    println!("{}", request);
    let lines: Vec<&str> = request.split("\r\n").collect();

    // Parse "POST /home HTTP/1.1"
    let resource = lines[0];
    let resource_parts: Vec<_> = resource.split(" ").collect();
    request_object.method = Some(*resource_parts.get(0).unwrap());
    request_object.url = Some(*resource_parts.get(1).unwrap());
    request_object.http_version = Some(*resource_parts.get(2).unwrap());

    // Parse headers starting on line 1 until newline
    let header_end = lines.iter().position(|l| *l == "").expect("failed to find first blank");
    let headers = &lines[1..header_end];

    let mut http_headers = Vec::new();
    for h in headers {
        let v: Vec<&str> = h.split(":").collect();
        http_headers.push(HttpHeader {
            key: v.get(0).expect("no key").to_lowercase(),
            value: v.get(1).expect("no value").trim().to_string(),
        })
    }
    request_object.headers = Some(&http_headers);

    // Parse the body
    let body = &lines[header_end + 1..];
    if body.len() > 0 {
        // TODO: Read Content-Type header application/x-www-form-urlencoded
        let body_content = body[0];
        request_object.body = Some(body[0].to_string());
        let data: Vec<&str> = body_content.split("&").collect();
        println!("{:?}", data);
    }

    println!("{:?}", request_object);
    let mut gzip_support = false;
    for h in http_headers.iter() {
        if h.key == "accept-encoding" {
            if h.value.contains("gzip") {
                gzip_support = true;
            }
        }
    }

    let mut method = HttpMethod::GET;
    if let Some(m) = request_object.method {
        if m == "GET" {
            method = HttpMethod::GET;
        } else if m == "POST" {
            method = HttpMethod::POST;
        }
    }

    let route = HttpRoute(method, &request_object.url.unwrap());
    let func_ref = {
        let func_ref_guard = routes.lock().unwrap();
        func_ref_guard.get(&route).unwrap().clone()
    };
    let contents = func_ref(&request_object);

    let final_content: Vec<u8>;
    if gzip_support {
        let mut e = GzEncoder::new(Vec::new(), Compression::best());
        e.write_all(contents.as_bytes()).expect("failed to compress with gzip");
        final_content = e.finish().expect("failed to finish compression");
    } else {
        final_content = contents.into();
    }

    let mut response_writer = BufWriter::new(Vec::new());
    response_writer.write("HTTP/1.1 200 OK\r\n".as_bytes()).expect("failed to write to buffer");
    if gzip_support {
        response_writer.write("Content-Encoding: gzip\r\n".as_bytes()).expect("failed");
    }
    response_writer.write(format!("Content-Length: {}\r\n\r\n", final_content.len()).as_bytes()).expect("failed");
    response_writer.write(&final_content).expect("failed to write to buffer");
    stream.write(response_writer.buffer()).expect("failed to respond");
}
