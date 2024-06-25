use std::net::TcpListener;
use std::net::TcpStream;
use std::io::Write;
use std::io::BufReader;
use std::io::BufRead;
use std::io;
use std::fs;

static FILES_DIR: &str = "./public";

fn main() {
    println!("Starting minimal-server.");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_connection(stream)
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Request: {http_request:#?}");

    let request_line = http_request[0].clone();
    let target_path = request_line.split(" ").collect::<Vec<_>>()[1];

    let response = get_path_content(target_path, http_request);

    println!("Response: {response}");

    let _ = stream.write_all(response.as_bytes());
    stream.flush().unwrap();
}

fn get_path_content(target_path: &str, _http_request: Vec<String>) -> String {
    if target_path == "/" {
        let path = FILES_DIR.to_owned() + &"/base/server.ac".to_string();

        match read_file(path) {
            Ok(content) => {
                let file_content_len = content.len();
                process_server_file(content);
            },
            Err(e) => {
                println!("Error reading file: {}", e);
            },
        }
    }

    if target_path == "/" {
        let path = FILES_DIR.to_owned() + &"/base/main.html".to_string();
        let mut content_type = "text/html";

        println!("File path: {path}");

        if path.ends_with(".html") {
            content_type = "text/html";
        } else if path.ends_with(".js") {
            content_type = "text/javascript";
        } else if path.ends_with(".css") {
            content_type = "text/css";
        }

        match read_file(path) {
            Ok(contents) => {
                let file_content_len = contents.len();
                return format!("HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {file_content_len}\r\n\r\n{contents}");
            },
            Err(e) => {
                println!("Error reading file: {}", e);
            },
        }
    }

    return "HTTP/1.1 404 Not Found\r\n\r\n".to_string();
}

fn read_file(file_path: String) -> io::Result<String> {
    fs::read_to_string(file_path)
}

fn process_server_file(content: String) {
    let lines: Vec<_> = content.lines().collect();
    println!("Server action: {lines:#?}");
}