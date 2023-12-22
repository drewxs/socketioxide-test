mod state;

use axum::{routing::get, Router};
use chrono::Utc;
use socketioxide::{
    extract::{Data, SocketRef, State},
    SocketIo,
};
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing_subscriber::FmtSubscriber;

use state::{Message, MessageStore, Messages};

async fn on_connect(socket: SocketRef) {
    info!("Socket connected: {}", socket.id);

    socket.on(
        "join",
        |socket: SocketRef, Data::<String>(room_id), store: State<MessageStore>| async move {
            info!("{} joined room {}", socket.id, room_id);
            let _ = socket.leave_all();
            let _ = socket.join(room_id.clone());
            let messages = store.get(&room_id).await;
            let _ = socket.emit("messages", Messages { messages });
        },
    );

    socket.on(
        "message",
        |socket: SocketRef, Data::<Message>(data), store: State<MessageStore>| async move {
            info!("Received message: {:?}", data);

            let res = Message {
                room_id: data.room_id.clone(),
                user_id: data.user_id.clone(),
                text: data.text,
                timestamp: Utc::now(),
            };

            store.insert(res.clone()).await;

            let _ = socket.within(data.room_id).emit("message", res);
        },
    );

    socket.on("typing", |socket: SocketRef, Data::<String>(user_id)| {
        socket.broadcast().emit("{} is typing", user_id).ok();
    });

    socket.on(
        "stop typing",
        |socket: SocketRef, Data::<String>(user_id)| {
            socket.broadcast().emit("{} stopped typing", user_id).ok();
        },
    );
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing::subscriber::set_global_default(FmtSubscriber::default())?;

    let messages = MessageStore::default();

    let (layer, io) = SocketIo::builder().with_state(messages).build_layer();

    io.ns("/", on_connect);

    let app = Router::new().route("/", get(|| async { "hello" })).layer(
        ServiceBuilder::new()
            .layer(CorsLayer::permissive())
            .layer(layer),
    );

    info!("Starting server");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
