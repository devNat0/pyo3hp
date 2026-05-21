mod pyo3hp;
use pyo3hp::Pyo3HP;
use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use std::net::SocketAddr;
use std::path::Path;

const PROJECT_ROOT: &str = env!("CARGO_MANIFEST_DIR");
const APP_ROOT: &str = "python_app/";


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr: SocketAddr = ([127, 0, 0, 1], 3000).into();

    let listener = TcpListener::bind(addr).await?;
    println!("Listening on http://{}", addr);

    let python_app_path = Path::new(PROJECT_ROOT).join(APP_ROOT);
    let pyo3hp = Pyo3HP::new(python_app_path.to_string_lossy().to_string());

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let pyo3hp_clone = pyo3hp.clone();
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new().serve_connection(io, pyo3hp_clone).await {
                println!("Failed to serve connection: {:?}", err);
            }
        });
    }
}

