use std::net::TcpListener;
use std::net::TcpStream;
use std::io::Write;
use std::io::BufReader;
use std::io::BufRead;
use std::io;
use std::fs;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::path::Path;

mod endpoint;

static ENDPOINTS_LIST: Lazy<Mutex<Vec<endpoint::Endpoint>>> = Lazy::new(|| Mutex::new(Vec::new()));

static FILES_DIR: &str = "./public";
static SERVER_DIR: &str = "./server";

fn main() {
    println!("Starting minimal-server.");

    load_endpoints();

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

fn load_endpoints() {
    let path = SERVER_DIR;

    let mut data = ENDPOINTS_LIST.lock().unwrap();

    match list_files_in_directory(path) {
        Ok(files) => {
            for file in files {
                match read_file(SERVER_DIR.to_owned() + "/" + &file) {
                    Ok(content) => {
                        let mut content_lines: Vec<String> = content.lines().map(|line| line.to_string()).collect();
                        let content_split: Vec<_> = content_lines[0].split(" ").collect();
                        let method = content_split[0].to_string();
                        let path = content_split[1].to_string();

                        content_lines.remove(0);

                        (*data).push(endpoint::Endpoint {
                            file: file,
                            path: path,
                            method: method,
                            content: content_lines,
                        });
                    },
                    Err(e) => {
                        println!("Error reading file: {}", e);
                    },
                }
            }
        },
        Err(e) => println!("Error reading directory: {}", e),
    }
}

fn list_files_in_directory<P: AsRef<Path>>(path: P) -> io::Result<Vec<String>> {
    let mut file_names = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy().into_owned();
        file_names.push(file_name);
    }

    Ok(file_names)
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
    let endpoint_list = ENDPOINTS_LIST.lock().unwrap();

    for endpoint in &*endpoint_list {
        if target_path == endpoint.path {
            let endpoint_content = endpoint.content.clone();
            return process_server_file(endpoint_content);
        }
    }

    if target_path.ends_with(".html") || target_path.ends_with(".js") || target_path.ends_with(".css") {
        return process_file(target_path.to_string());
    }

    return "HTTP/1.1 404 Not Found\r\n\r\n".to_string();
}

fn read_file(file_path: String) -> io::Result<String> {
    fs::read_to_string(file_path)
}

fn process_file(target_file: String) -> String {
    let path = FILES_DIR.to_owned() + "/" + &target_file;
    let mut content_type = "text/html";

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

    return "HTTP/1.1 404 Not Found\r\n\r\n".to_string();
}

fn process_server_file(server_actions: Vec<String>) -> String {
    for action in &server_actions {
        if action.starts_with("send_file") {
            let target_file = action.split(" ").collect::<Vec<_>>()[1].to_string();
            return process_file(target_file);
        }
    }

    return "HTTP/1.1 404 Not Found\r\n\r\n".to_string();
}