use pyo3::prelude::*;
use std::ffi::CString;
use std::path::PathBuf;
use std::{fs, path::Path};

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::service::Service;
use hyper::{body::Incoming as IncomingBody, Request, Response};
use std::future::Future;
use std::pin::Pin;

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

// returns python stdout
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

// https://github.com/hyperium/hyper/blob/master/examples/service_struct_impl.rs
#[derive(Debug, Clone)]
pub struct Pyo3HP {
    python_app_path: String
}

impl Pyo3HP {
    pub fn new(python_app_path: String) -> Self {
        Self {python_app_path}
    }

    // Combine python app path with request uri
    fn get_file_path(&self, path: &str) -> PathBuf {
        let index = "index.py";
        let url = if path.ends_with('/') {
            format!("{}{index}", path)
        } else {
            path.to_string()
        };

        let url = url.strip_prefix('/').unwrap();
        let py_path = Path::new(&self.python_app_path).join(url);
        if !py_path.is_file() {
            println!("file doesnt exist {}", py_path.display());
        }
        py_path
    }
}

impl Service<Request<IncomingBody>> for Pyo3HP {
    type Response = Response<Full<Bytes>>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<IncomingBody>) -> Self::Future {
        fn mk_response(s: String) -> Result<Response<Full<Bytes>>, hyper::Error> {
            Ok(Response::new(Full::new(Bytes::from(s))))
        }
        let res = mk_response(
            run_python_file(
                self.get_file_path(req.uri().path())
            )
        );
        Box::pin(async { res })
    }
}