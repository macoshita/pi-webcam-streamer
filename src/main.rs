use axum::{
    response::Html,
    routing::get,
    Router,
};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    // Build our application with a single route
    let app = Router::new().route("/", get(index_handler));

    // Run it with hyper on localhost:8080
    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("Server running on http://0.0.0.0:8080");
    axum::serve(listener, app).await.unwrap();
}

async fn index_handler() -> Html<&'static str> {
    Html(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Webcam Stream</title>
    <style>
        body { font-family: Arial, sans-serif; max-width: 800px; margin: 50px auto; text-align: center; }
        h1 { color: #333; }
        img { max-width: 100%; border: 2px solid #333; border-radius: 8px; }
    </style>
</head>
<body>
    <h1>Webcam Stream</h1>
    <img src="/stream" alt="Webcam Stream">
</body>
</html>"#,
    )
}
