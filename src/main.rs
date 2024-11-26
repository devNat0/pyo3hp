#[macro_use] extern crate rocket;
use std::{fs, path::Path};
use pyo3::prelude::*;
use rocket::response::content::RawHtml;

const APP_ROOT: &str = "python_app/";
const PROJECT_ROOT: &str = env!("CARGO_MANIFEST_DIR");

#[pyclass]
struct LoggingStdout {
    buffer: String
}

#[pymethods]
impl LoggingStdout {
    fn write(&mut self, data: &str) {
        self.buffer.push_str(data)
    }
}

fn parse(url: &str) -> String {
    let py_path = Path::new(PROJECT_ROOT)
        .join(APP_ROOT)
        .join(url.to_owned() + ".py");
    let py_app = fs::read_to_string(py_path).unwrap();
    Python::with_gil(|py| {
        let obj = Bound::new(py, LoggingStdout{buffer: "".to_string()}).unwrap();
        let sys = py.import_bound("sys").unwrap();
        sys.setattr("stdout", obj.clone().into_py(py)).unwrap();
        PyModule::from_code_bound(py, &py_app, "", "").unwrap();
        let gad_rs: PyRefMut<'_, LoggingStdout> = obj.extract().unwrap();
        gad_rs.buffer.clone()
    })
}

#[get("/")]
fn index() -> RawHtml<String> {
    // TODO: redirect to /index.py
    RawHtml(parse("index"))
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
}
