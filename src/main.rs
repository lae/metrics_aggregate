extern crate unix_socket;
extern crate core_mini_http;
extern crate chunked_transfer;
#[macro_use]
extern crate nom;

use unix_socket::UnixStream;
use core_mini_http::*;
use chunked_transfer::Decoder as ChunkedResponseDecoder;

use std::process;
use std::io::prelude::*;
use std::env;
use std::path::Path;
use std::thread;
use std::sync::mpsc;

mod nom_parsers;
use nom_parsers::parse;

static REQUEST_BODY: &'static [u8] =
    b"GET /metrics HTTP/1.1\r\n\
    Connection: close\r\n\
    Host: threatmatrix.eng.fireeye.com\r\n\r\n";


fn main() {
    let args = env::args().collect::<Vec<String>>();
    if args.len() != 3 {
        println!(
            "{} [socket directory] [socket count]",
            &args[0]
        );
        process::exit(1);
    }
    let socket_dir = args[1].to_string();
    let socket_count: u8 = match args[2].parse::<u8>() {
        Ok(c) => c,
        Err(_) => {
            println!("Socket count must be a number (given: {}).", &args[2]);
            process::exit(1);
        }
    };
    aggregate(&socket_dir, socket_count);
}

fn aggregate(directory: &str, count: u8) {
    println!("directory: {}\nsocket count: {}", directory, count);
    let (tx, rx) = mpsc::channel();

    for socket_number in 0..count {
        let path = validate_socket_path(directory, socket_number).ok().unwrap();
        let tx = tx.clone();
        thread::spawn(move || {
            let metrics = get_metrics_from_socket(&path);
            match metrics {
                Ok(m) => {
                    parse(m.as_bytes());
                    //println!("{}", m),
                },
                Err(e) => println!("{}, {:?}", path, e)
            }
            let _ = tx.send(());
        });
    }

    for _ in 0..count {
        let _ = rx.recv();
    }
}

fn get_metrics_from_socket(socket: &str) -> std::io::Result<String> {
    let mut conn = match UnixStream::connect(socket) {
        Ok(conn) => conn,
        Err(e) => return Err(e)
    };
    try!(conn.write_all(REQUEST_BODY));

    // parse the response in batches...
    let mut parser = HttpParser::new_response();
    loop {
        let mut buf = [0; 65536];
        let read_bytes = try!(conn.read(&mut buf));
        if read_bytes == 0 {
            // ...until nothing's left to parse
            break;
        }
        /*
            this function probably needs to be refactored a bit to not use
            std::io::Result, or handle the following error and the error type
            from to_vec further below differently. no panic plz
        */
        match parser.parse_bytes(&buf[..read_bytes]) {
            Ok(_) => continue,
            Err(_) => panic!("fuck")
        }
        if parser.read_how_many_bytes() == 0 {
            break;
        }
    }

    let http_msg = parser.get_response().unwrap();
    if http_msg.headers["Transfer-Encoding"] == "chunked" {
        // Now we want to parse out the chunk metadata here basically
        let mut response = String::new();
        let mut decoder = ChunkedResponseDecoder::new(&http_msg.body[..]);
        try!(decoder.read_to_string(&mut response));
        Ok(response)
    } else {
        // Otherwise just return the body as a string as-is
        let response = match String::from_utf8(http_msg.body.to_vec()) {
            Ok(s) => s,
            Err(_) => panic!("fuck")
        };
        Ok(response)
    }
}

fn validate_socket_path(dir: &str, num: u8) -> Result<String, String> {
    // This only gives us a relatively clean path string, for now
    let path = Path::new(dir).join(format!("{}.socket", num)).to_str()
                            .unwrap_or("").to_string();
    /*
    // TODO: use FileTypeExt::is_socket() instead for earlier termination when
    // its API is stable. (https://doc.rust-lang.org/stable/std/os/unix/fs/trait.FileTypeExt.html)
    let path_metadata = match fs::metadata(&path) {
        Ok(md) => md,
        Err(e) => return Err(e.to_string())
    };
    if path_metadata.file_type().is_socket() {
        Ok(path)
    } else {
        Err(format!("{} isn't a socket.", path))
    }
    */
    Ok(path)
}
