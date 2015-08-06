extern crate unix_socket;
extern crate http_muncher;
extern crate regex;

use unix_socket::UnixStream;
use http_muncher::{Parser, ParserHandler};
//use regex::Regex;

use std::io::prelude::*;
use std::env;
use std::thread;
use std::sync::mpsc;


struct MetricsParser {
    lines: Vec<String>
}

impl ParserHandler for MetricsParser {
    fn on_body(&mut self, s: &[u8]) -> bool {
        let body = std::str::from_utf8(s).unwrap().to_string();
        let lines: Vec<&str> = body.split("\n").collect();
        for line in lines {
            self.lines.push(line.to_string());
        }
        true
    }
}
/*
struct MetricHistogram {
    desc_help: String,
    desc_type: String,
}

struct something;


struct Counter {
    value: f64,
}*/

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!(
            "{} [socket directory] [socket count]",
            args[0]
        );
    }
    else {
        let socket_dir = args[1].parse().unwrap();
        let socket_count: u8 = args[2].parse().unwrap();
        aggregate(socket_dir, socket_count);
    }
}

fn aggregate(directory: String, count: u8) {
    println!("directory: {}\nsocket count: {}", directory, count);
    let (tx, rx) = mpsc::channel();

    for socket_number in 0..count {
        let socket_path = format!(
            "{}/{}.socket",
            directory,
            socket_number
        ).to_string();
        let tx = tx.clone();
        thread::spawn(move || {
            poll_socket(socket_path);
            let _ = tx.send(());
        });
    }

    for _ in 0..count {
        let _ = rx.recv();
    }

}

fn poll_socket(path: String) {
    let host = "threatmatrix.eng.fireeye.com";

    let mut metrics_parser = Parser::response(MetricsParser {
        lines: Vec::new()
    });

    let metrics_request =  format!(
        "GET /metrics HTTP/1.1\r\n\
        Content-Type: text/plain\r\n\
        Host: {host}\r\n\r\n",
        host = &host);

    let mut stream = UnixStream::connect(&path).unwrap();
    stream.write_all(metrics_request.as_bytes()).unwrap();

    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();

    metrics_parser.parse(response.as_bytes());

    let ref metrics = metrics_parser.get().lines;

    println!(
        "path={path} first_line={first_line:?}",
        path = path,
        first_line = metrics[0]
    );

}