use pyo3::prelude::*;
use crate::models::ContentAnalysis;

#[pyclass]
pub struct TextAnalyzer {
    // 内部状态
}

#[pymethods]
impl TextAnalyzer {
    #[new]
    fn new() -> Self {
        Self {}
    }

    #[pyo3(signature = (text, api_key="demo_key"))]
    fn analyze_text(
        &self,
        text: &str,
        api_key: &str
    ) -> PyResult<ContentAnalysis> {
        // 基本的本地分析逻辑
        let mut violence_score: f32 = 0.0;
        if text.to_lowercase().contains("威胁") { violence_score += 0.5; }
        if text.to_lowercase().contains("讨厌") { violence_score += 0.3; }
        if text.to_lowercase().contains("恨") { violence_score += 0.4; }
        if text.to_lowercase().contains("滚") { violence_score += 0.4; }
        if text.to_lowercase().contains("垃圾") { violence_score += 0.3; }

        let violence_score = violence_score.min(1.0);

        let violence_type = if violence_score > 0.7 {
            Some("harassment".to_string())
        } else {
            None
        };

        Ok(ContentAnalysis::new(
            "text-1".to_string(),
            "text".to_string(),
            violence_score.into(),
            violence_type,
            0.85,
            None,
            None,
            None,
            vec![],
            "{}".to_string(),
        ))
    }
}