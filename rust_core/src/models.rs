use pyo3::prelude::*;
use serde::{Serialize, Deserialize};

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentAnalysis {
    #[pyo3(get, set)]
    pub content_id: String,

    #[pyo3(get, set)]
    pub content_type: String,

    #[pyo3(get, set)]
    pub violence_score: f64,

    #[pyo3(get, set)]
    pub violence_type: Option<String>,

    #[pyo3(get, set)]
    pub confidence_score: f64,

    #[pyo3(get, set)]
    pub sentiment: Option<i32>,  // 增加情感类型: 0-消极，1-中性，2-积极

    #[pyo3(get, set)]
    pub positive_prob: Option<f64>,  // 增加积极概率

    #[pyo3(get, set)]
    pub negative_prob: Option<f64>,  // 增加消极概率

    #[pyo3(get, set)]
    pub keywords: Vec<String>,  // 增加关键词列表

    #[pyo3(get, set)]
    pub metadata: String, // JSON string
}

#[pymethods]
impl ContentAnalysis {
    #[new]
    #[pyo3(signature = (
        content_id,
        content_type,
        violence_score,
        violence_type=None,
        confidence_score=0.0,
        sentiment=None,
        positive_prob=None,
        negative_prob=None,
        keywords=vec![],
        metadata=String::from("{}")
    ))]
    pub fn new(
        content_id: String,
        content_type: String,
        violence_score: f64,
        violence_type: Option<String>,
        confidence_score: f64,
        sentiment: Option<i32>,
        positive_prob: Option<f64>,
        negative_prob: Option<f64>,
        keywords: Vec<String>,
        metadata: String,
    ) -> Self {
        Self {
            content_id,
            content_type,
            violence_score,
            violence_type,
            confidence_score,
            sentiment,
            positive_prob,
            negative_prob,
            keywords,
            metadata,
        }
    }

    fn to_json(&self) -> PyResult<String> {
        Ok(serde_json::to_string(self).unwrap())
    }

    fn is_violent(&self) -> bool {
        self.violence_score > 0.7
    }

    // 新增方法，基于百度API结果判断是否为负面内容
    fn is_negative(&self) -> bool {
        if let Some(sentiment) = self.sentiment {
            return sentiment == 0; // 0表示消极
        }

        // 如果没有百度API结果，回退到自己的判断
        self.violence_score > 0.5
    }
}