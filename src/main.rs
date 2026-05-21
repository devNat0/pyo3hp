use pyo3::prelude::*;
use std::ffi::CString;
use std::path::PathBuf;
use std::{fs, path::Path};

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::Service;
use hyper::{body::Incoming as IncomingBody, Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

#[pyclass]
struct LoggingStdout {
    buffer: String,
}

#[pymethods]
impl LoggingStdout {
    fn write(&mut self, data: &str) {
        self.buffer.push_str(data)
    }
}

const APP_ROOT: &str = "python_app/";
const PROJECT_ROOT: &str = env!("CARGO_MANIFEST_DIR");

fn get_file_path(path: &str) -> PathBuf {
    let index = "index.py";
    let url = if path.ends_with('/') {
        format!("{}{index}", path)
    } else {
        path.to_string()
    };

    let url = url.strip_prefix('/').unwrap();
    let py_path = Path::new(PROJECT_ROOT).join(APP_ROOT).join(url);
    if !py_path.is_file() {
        println!("file doesnt exist {}", py_path.display());
    }
    println!("{}", py_path.display());
    println!("{}", APP_ROOT);
    py_path
}

fn run_python_file(file_path: PathBuf) -> String {
    let code = fs::read_to_string(file_path).unwrap();
    Python::attach(|py| {
        let obj = Bound::new(
            py,
            LoggingStdout {
                buffer: "".to_string(),
            },
        )
        .unwrap();
        let sys = py.import("sys").unwrap();
        let _ = sys.setattr("stdout", &obj);
        PyModule::from_code(
            py,
            CString::new(code).unwrap().as_c_str(),
            c"",
            c"",
        )
        .unwrap();
        let val: PyRefMut<'_, LoggingStdout> = obj.extract().unwrap();
        val.buffer.clone()
    })
}

type Counter = i32;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr: SocketAddr = ([127, 0, 0, 1], 3000).into();

    let listener = TcpListener::bind(addr).await?;
    println!("Listening on http://{}", addr);

    let svc = Svc {
        counter: Arc::new(Mutex::new(0)),
    };

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let svc_clone = svc.clone();
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new().serve_connection(io, svc_clone).await {
                println!("Failed to serve connection: {:?}", err);
            }
        });
    }
}

// https://github.com/hyperium/hyper/blob/master/examples/service_struct_impl.rs
#[derive(Debug, Clone)]
struct Svc {
    counter: Arc<Mutex<Counter>>,
}

impl Service<Request<IncomingBody>> for Svc {
    type Response = Response<Full<Bytes>>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<IncomingBody>) -> Self::Future {
        fn mk_response(s: String) -> Result<Response<Full<Bytes>>, hyper::Error> {
            Ok(Response::new(Full::new(Bytes::from(s))))
        }
        let res = mk_response(
            run_python_file(
                get_file_path(req.uri().path())
            )
        );
        Box::pin(async { res })
    }
}
