use pyo3::prelude::*;

mod models;
mod analyzer;
mod rules;
mod baidu_api;

#[pymodule]
fn violence_governance_core(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<models::ContentAnalysis>()?;
    m.add_class::<analyzer::TextAnalyzer>()?;
    m.add_class::<rules::GovernanceAction>()?;
    m.add_function(wrap_pyfunction!(rules::determine_action, m)?)?;
    Ok(())
}