use tokio::fs::read;
use tokio::io::{ AsyncBufReadExt, AsyncWriteExt, BufReader, Result };
use tokio::net::{ TcpListener, TcpStream };

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878").await?;

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("[+] Connction from {}", addr);

        tokio::spawn(async move {
            handle_connection(socket).await;
        });
    }
}

async fn handle_connection(stream: TcpStream) {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();

    if reader.read_line(&mut line).await.unwrap_or(0) == 0 {
        println!("Connection closed before request");
        return;
    }

    println!("Recevied request: {}", line.trim_end());

    loop {
        let mut header = String::new();
        let bytes = reader.read_line(&mut header).await.unwrap_or(0);
        if bytes == 0 || header == "\r\n" {
            break;
        }
    }

    let stream = reader.into_inner(); // get that TcpStream back!
    if let Some((method, path)) = parse_request_line(&line) {
        if let Err(e) = respond_to_request(method, path, stream).await {
            eprintln!("Error handling request: {}", e);
        }
    } else {
        send_status(stream, 400, "Bad Request", "text/html", b"Bad Request").await.unwrap();
    }
}

async fn respond_to_request(method: &str, path: &str, stream: TcpStream) -> Result<()> {
    if method != "GET" {
        return send_status(
            stream,
            405,
            "Method Not Allowed",
            "text/html",
            b"Only GET is supported"
        ).await;
    }
    let error_content = read("out/404.html").await.unwrap_or_else(|_| b"404 Not Found".to_vec());
    let sanitized_path = if path == "/" {
        "index.html".to_string()
    } else {
        path.trim_start_matches('/').to_string()
    };

    if sanitized_path.contains("..") {
        return send_status(stream, 404, "Not Found", "text/html", &error_content).await;
    }

    let full_path = format!("out/{}", sanitized_path);

    match read(&full_path).await {
        Ok(content) => {
            let content_type = get_content_type(&full_path);
            send_status(stream, 200, "OK", content_type, &content).await?;
        }
        Err(_) => {
            send_status(stream, 404, "Not Found", "text/html", &error_content).await?;
        }
    }

    Ok(())
}

fn parse_request_line(line: &str) -> Option<(&str, &str)> {
    let mut parts = line.split_whitespace();

    let method = parts.next()?;
    let path = parts.next()?;
    let _version = parts.next()?;

    Some((method, path))
}

async fn send_status(
    mut stream: TcpStream,
    code: u16,
    reason: &str,
    content_type: &str,
    body: &[u8]
) -> Result<()> {
    let response = format!(
        "HTTP/1.1 {} {}\r\n\
        Content-Length: {}\r\n\
        Content-Type: {}\r\n\
        Connection: close\r\n\
        \r\n",
        code,
        reason,
        body.len(),
        content_type
    );
    stream.write_all(response.as_bytes()).await?;
    stream.write_all(body).await?;
    stream.shutdown().await?;
    Ok(())
}

fn get_content_type(path: &str) -> &'static str {
    if path.ends_with(".html") {
        "text/html; charset=UTF-8"
    } else if path.ends_with(".css") {
        "text/css"
    } else if path.ends_with(".js") {
        "application/javascript"
    } else if path.ends_with(".json") {
        "application/json"
    } else if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
        "image/jpeg"
    } else if path.ends_with(".gif") {
        "image/gif"
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else if path.ends_with(".ico") {
        "image/x-icon"
    } else if path.ends_with(".woff2") {
        "font/woff2"
    } else if path.ends_with(".woff") {
        "font/woff"
    } else if path.ends_with(".ttf") {
        "font/ttf"
    } else if path.ends_with(".otf") {
        "font/otf"
    } else {
        "application/octet-stream"
    }
}
