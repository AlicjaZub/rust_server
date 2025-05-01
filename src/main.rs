use tokio::fs::read;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Result};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878").await?;

    loop {
        let (socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            handle_connection(socket).await;
        });
    }
}

async fn handle_connection(mut stream: TcpStream) {
    let mut reader = BufReader::new(&mut stream);
    let mut line = String::new();

    match reader.read_line(&mut line).await {
        Ok(0) => {
            println!("No request received (connection closed).");
        }
        Ok(_) => {
            println!("Received request: {}", line.trim_end());
            if let Err(e) = respond_to_request(&line, &mut stream).await {
                eprintln!("Error handling request: {}", e);
            }
        }
        Err(e) => {
            println!("Error reading from stream: {}", e);
        }
    }
}

async fn respond_to_request(line: &str, stream: &mut TcpStream) -> Result<()> {
    let request_path = line.split_whitespace().nth(1).unwrap_or("/");
    let sanitized_path = request_path.strip_prefix("/").unwrap_or(request_path);
    let full_path = format!("out/{}", sanitized_path);

    let content = if let Ok(data) = read(&full_path).await {
        data
    } else {
        read("out/404.html").await.unwrap()
    };
    let content_type = get_content_type(&full_path);

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: {}\r\n\r\n",
        content.len(),
        content_type
    );

    stream.write_all(response.as_bytes()).await?;
    stream.write_all(&content).await?;

    Ok(())
}

fn get_content_type(path: &str) -> &'static str {
    if path.ends_with(".html") {
        "text/html"
    } else if path.ends_with(".css") {
        "text/css"
    } else if path.ends_with(".js") {
        "application/javascript"
    } else if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
        "image/jpeg"
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else if path.ends_with(".json") {
        "application/json"
    } else {
        "application/octet-stream"
    }
}
