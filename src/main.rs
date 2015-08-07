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

static REQUEST_BODY: &'static [u8] =
    b"GET /metrics HTTP/1.1\r\n\
    Connection: close\r\n\
    Host: threatmatrix.eng.fireeye.com\r\n\r\n";


struct MetricsParser {
    lines: Vec<String>
}

impl ParserHandler for MetricsParser {
    fn on_body(&mut self, s: &[u8]) -> bool {
        let body = std::str::from_utf8(s).ok().unwrap().to_string();
        for line in body.lines() {
            match line.parse::<String>() {
                Ok(..) => self.lines.push(line.to_string()),
                Err(..) => {}
            }
        }
        true
    }
}

struct MetricHistogram {
    desc_help: String,
    desc_type: String,
}

struct Counter {
    value: f64,
}


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
    println!("idk man");

}

fn poll_socket(path: String) {
    let mut metrics_parser = Parser::response(MetricsParser {
        lines: Vec::new()
    });

    let mut stream = UnixStream::connect(&path).unwrap();
    let _ = stream.write(REQUEST_BODY).unwrap();

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
