use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use pyo3::PyObject;
use regex::Regex;
use rayon::prelude::*;
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
        Regex::new(pattern)
            .map(|regex| self.patterns.push((regex, event_type.to_string())))
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid regex pattern: {}", e)))
    }

    fn add_patterns(&mut self, patterns: &PyList) -> PyResult<()> {
        for item in patterns.iter() {
            let (regex, event_type): (String, String) = item.extract()?;
            self.add_pattern(&regex, &event_type)?;
        }
        Ok(())
    }

    fn register_format(&mut self, format_name: &str, patterns: &PyList) -> PyResult<()> {
        let mut format_patterns = Vec::new();
        for item in patterns.iter() {
            let (regex, event_type): (String, String) = item.extract()?;
            format_patterns.push((regex, event_type));
        }
        self.format_configs.insert(format_name.to_string(), format_patterns);
        Ok(())
    }

    fn activate_format(&mut self, format_name: &str) -> PyResult<()> {
        let cloned_patterns = self
            .format_configs
            .get(format_name)
            .cloned()
            .ok_or_else(|| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Format {} not found",
                    format_name
                ))
            })?;
    
        self.patterns.clear();
    
        for (regex, event_type) in cloned_patterns {
            self.add_pattern(&regex, &event_type)?;
        }
    
        Ok(())
    }

    fn match_message<'a>(&self, message: &'a str) -> Option<(String, PyObject)> {
        Python::with_gil(|py| {
            for (regex, event_type) in &self.patterns {
                if let Some(captures) = regex.captures(message) {
                    let dict = PyDict::new(py);
                    for name in regex.capture_names().flatten() {
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

    fn match_messages_batch(&self, messages: &PyList) -> PyResult<Vec<Option<(String, PyObject)>>> {
        let patterns = self.patterns.clone();
        let string_messages: Vec<String> = messages.iter().map(|item| item.extract::<String>()).collect::<Result<_, _>>()?;

        let native_results: Vec<_> = string_messages
            .par_iter()
            .map(|message| {
                for (regex, event_type) in &patterns {
                    if let Some(captures) = regex.captures(message) {
                        let mut capture_data = Vec::new();
                        for name in regex.capture_names().flatten() {
                            if let Some(m) = captures.name(name) {
                                capture_data.push((name.to_string(), m.as_str().to_string()));
                            }
                        }
                        return Some((event_type.clone(), capture_data));
                    }
                }
                None
            })
            .collect();

        Python::with_gil(|py| {
            Ok(native_results
                .into_iter()
                .map(|result| {
                    result.map(|(event_type, capture_data)| {
                        let dict = PyDict::new(py);
                        for (name, value) in capture_data {
                            dict.set_item(name, value).ok()?;
                        }
                        Some((event_type, dict.to_object(py)))
                    }).flatten()
                })
                .collect())
        })
    }

    fn match_messages_batch_native(
        &self,
        messages: Vec<String>,
    ) -> Vec<Option<(String, Vec<(String, String)>)>> {
        let patterns = &self.patterns;
        if messages.len() >= 1000 {
            messages
                .into_par_iter()
                .map(|msg| match_one(&msg, patterns))
                .collect()
        } else {
            messages
                .into_iter()
                .map(|msg| match_one(&msg, patterns))
                .collect()
        }
    }

    fn get_available_formats(&self) -> Vec<String> {
        self.format_configs.keys().cloned().collect()
    }
}

fn match_one(
    message: &str,
    patterns: &[(Regex, String)],
) -> Option<(String, Vec<(String, String)>)> {
    for (regex, event_type) in patterns {
        if let Some(captures) = regex.captures(message) {
            let mut capture_data = Vec::with_capacity(4);
            for name in regex.capture_names().flatten() {
                if let Some(m) = captures.name(name) {
                    capture_data.push((name.to_string(), m.as_str().to_string()));
                }
            }
            return Some((event_type.clone(), capture_data));
        }
    }
    None
}

#[pymodule]
fn cgn_ec_rust_parsers(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<RegexMatcher>()?;
    Ok(())
}
