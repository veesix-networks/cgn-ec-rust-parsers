use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use pyo3::PyObject;
use regex::Regex;
use std::collections::HashMap;

#[pyclass]
struct RegexMatcher {
    patterns: Vec<(Regex, String)>,
    format_configs: HashMap<String, Vec<(String, String)>>,
}

#[pymethods]
impl RegexMatcher {
    #[new]
    fn new() -> Self {
        RegexMatcher {
            patterns: Vec::new(),
            format_configs: HashMap::new(),
        }
    }

    fn add_pattern(&mut self, pattern: &str, event_type: &str) -> PyResult<()> {
        match Regex::new(pattern) {
            Ok(regex) => {
                self.patterns.push((regex, event_type.to_string()));
                Ok(())
            }
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Invalid regex pattern: {}", e),
            )),
        }
    }

    fn add_patterns(&mut self, patterns: &PyList) -> PyResult<()> {
        for pattern_item in patterns.iter() {
            let tuple = unsafe { pattern_item.extract::<(String, String)>()? };
            self.add_pattern(&tuple.0, &tuple.1)?;
        }
        Ok(())
    }

    fn register_format(&mut self, format_name: &str, patterns: &PyList) -> PyResult<()> {
        let mut format_patterns = Vec::new();
        
        for pattern_item in patterns.iter() {
            let tuple = unsafe { pattern_item.extract::<(String, String)>()? };
            format_patterns.push((tuple.0, tuple.1));
        }
        
        self.format_configs.insert(format_name.to_string(), format_patterns);
        Ok(())
    }

    fn activate_format(&mut self, format_name: &str) -> PyResult<()> {
        if let Some(patterns) = self.format_configs.get(format_name).cloned() {
            self.patterns.clear();
    
            for (pattern, event_type) in patterns {
                self.add_pattern(&pattern, &event_type)?;
            }
            Ok(())
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Format {} not found", format_name),
            ))
        }
    }

    fn match_message<'a>(&self, message: &'a str) -> Option<(String, PyObject)> {
        Python::with_gil(|py| {
            for (regex, event_type) in &self.patterns {
                if let Some(captures) = regex.captures(message) {
                    // Create a dictionary for named groups
                    let dict = PyDict::new(py);
                    
                    // Get all named capture groups
                    for name in regex.capture_names().filter_map(|n| n) {
                        if let Some(m) = captures.name(name) {
                            dict.set_item(name, m.as_str()).unwrap();
                        }
                    }
                    
                    return Some((event_type.clone(), dict.to_object(py)));
                }
            }
            
            None
        })
    }
    
    fn get_available_formats(&self) -> Vec<String> {
        self.format_configs.keys().cloned().collect()
    }
}

#[pymodule]
fn cgn_ec_rust_parsers(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<RegexMatcher>()?;
    Ok(())
}