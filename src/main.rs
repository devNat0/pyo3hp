#[macro_use]
extern crate rocket;
use pyo3::{ffi::c_str, prelude::*};
use pyo3::types::IntoPyDict;
use rocket::response::content::RawHtml;
use std::ffi::CString;
use std::{fs, path::Path};

const APP_ROOT: &str = "python_app/";
const PROJECT_ROOT: &str = env!("CARGO_MANIFEST_DIR");

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

fn parse(url: &str) -> String {
    let py_path = Path::new(PROJECT_ROOT)
        .join(APP_ROOT)
        .join(url.to_owned() + ".py");
    let code = fs::read_to_string(py_path).unwrap();
    Python::attach(|py| {
        let obj = Bound::new(py, LoggingStdout{buffer: "".to_string()}).unwrap();
        let sys = py.import("sys").unwrap();
        let _ = sys.setattr("stdout", &obj);
        PyModule::from_code(py, CString::new(code).unwrap().as_c_str(), c"mycode.py", c"mycode").unwrap();
        let val: PyRefMut<'_, LoggingStdout> = obj.extract().unwrap();
        val.buffer.clone()
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
