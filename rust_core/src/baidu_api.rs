use reqwest::blocking::{Client, Response};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub expires_in: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentRequest {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentItem {
    pub sentiment: i32,         // 情感极性分类结果, 0:负向，1:中性，2:正向
    pub confidence: f64,        // 分类的置信度
    pub positive_prob: f64,     // 属于积极类别的概率
    pub negative_prob: f64,     // 属于消极类别的概率
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentResponse {
    pub items: Vec<SentimentItem>,
    #[serde(rename = "log_id")]
    pub log_id: u64,
}

// 自定义错误类型
pub struct ApiError {
    pub message: String,
}

impl From<reqwest::Error> for ApiError {
    fn from(error: reqwest::Error) -> Self {
        ApiError {
            message: error.to_string(),
        }
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(error: serde_json::Error) -> Self {
        ApiError {
            message: error.to_string(),
        }
    }
}

pub struct BaiduApiClient {
    client: Client,
    api_key: String,
    secret_key: String,
    token_info: Arc<Mutex<Option<(String, u64)>>>, // (token, expire_time)
}

impl BaiduApiClient {
    pub fn new(api_key: &str, secret_key: &str) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.to_string(),
            secret_key: secret_key.to_string(),
            token_info: Arc::new(Mutex::new(None)),
        }
    }

    pub fn get_access_token(&self) -> Result<String, ApiError> {
        // 检查缓存的token是否有效
        {
            let token_info = self.token_info.lock().unwrap();
            if let Some((token, expire_time)) = &*token_info {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                // Token未过期，直接返回
                if now < *expire_time {
                    return Ok(token.clone());
                }
            }
        }

        // 构建获取token的URL
        let url = format!(
            "https://aip.baidubce.com/oauth/2.0/token?grant_type=client_credentials&client_id={}&client_secret={}",
            self.api_key, self.secret_key
        );

        // 发送请求获取token
        let response = self.client.get(&url).send()?;

        if !response.status().is_success() {
            return Err(ApiError {
                message: format!("获取access_token失败: {}", response.status()),
            });
        }

        let result: TokenResponse = response.json()?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // 设置过期时间（提前5分钟过期）
        let expire_time = now + result.expires_in - 300;

        // 更新缓存
        {
            let mut token_info = self.token_info.lock().unwrap();
            *token_info = Some((result.access_token.clone(), expire_time));
        }

        Ok(result.access_token)
    }

    pub fn sentiment_analysis(&self, text: &str) -> Result<SentimentItem, ApiError> {
        // 获取access_token
        let access_token = self.get_access_token()?;

        // 构建API URL
        let url = format!(
            "https://aip.baidubce.com/rpc/2.0/nlp/v1/sentiment_classify?access_token={}&charset=UTF-8",
            access_token
        );

        // 构建请求头
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        // 构建请求体
        let request = SentimentRequest {
            text: text.to_string(),
        };

        // 发送请求
        let response = self.client
            .post(&url)
            .headers(headers)
            .json(&request)
            .send()?;

        if !response.status().is_success() {
            return Err(ApiError {
                message: format!("情感分析API调用失败: {}", response.status()),
            });
        }

        // 解析响应
        let result: SentimentResponse = response.json()?;

        if result.items.is_empty() {
            return Err(ApiError {
                message: "情感分析API返回空结果".to_string(),
            });
        }

        Ok(result.items[0].clone())
    }
}