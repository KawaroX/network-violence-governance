use pyo3::prelude::*;
use serde::{Serialize, Deserialize};
use crate::models::ContentAnalysis;

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceAction {
    #[pyo3(get, set)]
    pub action_type: String,

    #[pyo3(get, set)]
    pub severity: String,

    #[pyo3(get, set)]
    pub message: Option<String>,

    #[pyo3(get, set)]
    pub automated: bool,
}

#[pymethods]
impl GovernanceAction {
    #[new]
    fn new(
        action_type: String,
        severity: String,
        message: Option<String>,
        automated: bool,
    ) -> Self {
        Self {
            action_type,
            severity,
            message,
            automated,
        }
    }

    fn to_json(&self) -> PyResult<String> {
        Ok(serde_json::to_string(self).unwrap())
    }
}

#[pyfunction]
pub fn determine_action(analysis: &ContentAnalysis) -> GovernanceAction {
    // 使用violence_score和negative_prob（如果有）计算最终分数
    let final_score = if let Some(neg_prob) = analysis.negative_prob {
        (analysis.violence_score + neg_prob) / 2.0
    } else {
        analysis.violence_score
    };

    // 基于最终分数和暴力类型确定行动
    match (final_score, &analysis.violence_type) {
        (score, Some(vtype)) if score > 0.8 && vtype == "harassment" => {
            GovernanceAction::new(
                "remove".to_string(),
                "critical".to_string(),
                Some("该内容包含严重骚扰行为，已被系统移除".to_string()),
                true,
            )
        },
        (score, Some(vtype)) if score > 0.8 || vtype == "threat" => {
            GovernanceAction::new(
                "restrict".to_string(),
                "critical".to_string(),
                Some("该内容包含威胁或高风险内容，已被限制访问".to_string()),
                true,
            )
        },
        (score, _) if score > 0.7 => {
            GovernanceAction::new(
                "restrict".to_string(),
                "high".to_string(),
                Some("该内容可能包含有害信息，已被限制可见性".to_string()),
                true,
            )
        },
        (score, _) if score > 0.5 => {
            GovernanceAction::new(
                "warning".to_string(),
                "medium".to_string(),
                Some("请注意您的言论，避免伤害他人".to_string()),
                false,
            )
        },
        (score, _) if score > 0.3 => {
            GovernanceAction::new(
                "flag".to_string(),
                "low".to_string(),
                Some("该内容已被标记进行进一步审核".to_string()),
                false,
            )
        },
        _ => {
            GovernanceAction::new(
                "none".to_string(),
                "none".to_string(),
                None,
                false,
            )
        },
    }
}