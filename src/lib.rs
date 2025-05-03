use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use pyo3::PyObject;
use regex::Regex;
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

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
    
    fn match_messages_batch(&self, messages: &PyList) -> PyResult<Vec<Option<(String, PyObject)>>> {
        let patterns: Arc<Vec<(Regex, String)>> = Arc::new(self.patterns.clone());
        
        let string_messages: Vec<String> = messages.iter()
            .map(|item| item.extract::<String>())
            .collect::<Result<Vec<_>, _>>()?;
            
        let native_results: Vec<_> = string_messages
            .par_iter()
            .map(|message| {
                let mut matched = None;
                
                for (idx, (regex, event_type)) in patterns.iter().enumerate() {
                    if let Some(captures) = regex.captures(message) {
                        let mut capture_data = Vec::new();
                        for name in regex.capture_names().filter_map(|n| n) {
                            if let Some(m) = captures.name(name) {
                                capture_data.push((name.to_string(), m.as_str().to_string()));
                            }
                        }
                        
                        matched = Some((idx, event_type.clone(), capture_data));
                        break;
                    }
                }
                
                matched
            })
            .collect();
        
        Python::with_gil(|py| {
            Ok(native_results.into_iter().map(|result| {
                match result {
                    Some((_, event_type, capture_data)) => {
                        let dict = PyDict::new(py);
                        for (name, value) in capture_data {
                            if let Err(_) = dict.set_item(name, value) {
                                return None;
                            }
                        }
                        Some((event_type, dict.to_object(py)))
                    },
                    None => None,
                }
            }).collect())
        })
    }

    fn match_messages_batch_native(
        &self,
        messages: Vec<String>,
    ) -> Vec<Option<(String, Vec<(String, String)>)>> {
        let patterns: Arc<Vec<(Regex, String)>> = Arc::new(self.patterns.clone());

        messages
            .into_par_iter()
            .map(|message| {
                for (regex, event_type) in patterns.iter() {
                    if let Some(captures) = regex.captures(&message) {
                        let capture_data: Vec<(String, String)> = regex
                            .capture_names()
                            .filter_map(|n| n)
                            .filter_map(|name| {
                                captures.name(name).map(|m| (name.to_string(), m.as_str().to_string()))
                            })
                            .collect();

                        return Some((event_type.clone(), capture_data));
                    }
                }
                None
            })
            .collect()
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