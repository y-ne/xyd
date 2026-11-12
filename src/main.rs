use axum::{
    Json, Router,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::Html,
    routing::get,
};
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use serde::Serialize;
use std::io::{Read, Write};
use tokio::net::TcpListener;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(root))
        .route("/terminal", get(terminal_page))
        .route("/ws", get(ws_handler));

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("xyd daemon is listening on port 3000...");

    axum::serve(listener, app).await.unwrap();
}

#[derive(Serialize)]
struct HelloResponse {
    message: String,
}

async fn root() -> Json<HelloResponse> {
    let response = HelloResponse {
        message: "hello from xyd".to_string(),
    };
    Json(response)
}

async fn terminal_page() -> Html<&'static str> {
    Html(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/xterm@5.3.0/css/xterm.css" />
            <script src="https://cdn.jsdelivr.net/npm/xterm@5.3.0/lib/xterm.js"></script>
        </head>
        <body style="background: #000; margin: 0; padding: 10px;">
            <div id="terminal" style="height: 95vh; width: 95vw;"></div>
            <script>
                const term = new Terminal({ cursorBlink: true, theme: { background: '#000000' } });
                term.open(document.getElementById('terminal'));

                const ws = new WebSocket(`ws://${location.host}/ws`);

                ws.onmessage = (event) => term.write(event.data);
                term.onData(data => ws.send(data));
            </script>
        </body>
        </html>
    "#,
    )
}

async fn ws_handler(ws: WebSocketUpgrade) -> axum::response::Response {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    let pty_system = native_pty_system();

    let pty_pair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .unwrap();

    let mut cmd = CommandBuilder::new("su");
    cmd.env("TERM", "xterm-256color");
    let mut child = pty_pair.slave.spawn_command(cmd).unwrap();

    let mut reader = pty_pair.master.try_clone_reader().unwrap();
    let mut writer = pty_pair.master.take_writer().unwrap();

    let (tx, mut rx) = mpsc::channel::<String>(32);

    tokio::task::spawn_blocking(move || {
        let mut buf = [0u8; 8192];
        while let Ok(n) = reader.read(&mut buf) {
            if n == 0 {
                break;
            }
            if tx
                .blocking_send(String::from_utf8_lossy(&buf[..n]).into_owned())
                .is_err()
            {
                break;
            }
        }
    });

    loop {
        tokio::select! {
            Some(out) = rx.recv() => {
                if socket.send(Message::Text(out.into())).await.is_err() { break; }
            }
            Some(Ok(msg)) = socket.recv() => {
                if let Message::Text(text) = msg {
                    if writer.write_all(text.as_bytes()).is_err() { break; }
                }
            }
            else => break,
        }
    }

    drop(writer);
    let _ = child.kill();
    let _ = child.wait();
}
