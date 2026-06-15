//! 有道智云 API 客户端
//! 用于从有道API获取单词数据

use md5;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 有道API配置
#[derive(Clone)]
pub struct YoudaoConfig {
    pub app_key: String,
    pub app_secret: String,
    pub api_url: String,
}

impl Default for YoudaoConfig {
    fn default() -> Self {
        Self {
            app_key: String::new(),
            app_secret: String::new(),
            api_url: "https://openapi.youdao.com/api".to_string(),
        }
    }
}

/// 有道API单词查询响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YoudaoResponse {
    #[serde(rename = "query")]
    pub query: String,
    #[serde(rename = "errorCode")]
    pub error_code: String,
    #[serde(rename = "translation")]
    pub translation: Option<Vec<String>>,
    #[serde(rename = "basic")]
    pub basic: Option<YoudaoBasic>,
    #[serde(rename = "web")]
    pub web: Option<Vec<YoudaoWebItem>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YoudaoBasic {
    #[serde(rename = "phonetic")]
    pub phonetic: Option<String>,
    #[serde(rename = "us-phonetic")]
    pub us_phonetic: Option<String>,
    #[serde(rename = "uk-phonetic")]
    pub uk_phonetic: Option<String>,
    #[serde(rename = "wfs")]
    pub wfs: Option<Vec<YoudaoWf>>,
    #[serde(rename = "exam_type")]
    pub exam_type: Option<Vec<String>>,
    #[serde(rename = "trans")]
    pub translations: Option<Vec<YoudaoTrans>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YoudaoTrans {
    #[serde(rename = "word")]
    pub word: Option<String>,
    #[serde(rename = "trans")]
    pub trans: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YoudaoWf {
    #[serde(rename = "wf")]
    pub wf: YoudaoWfInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YoudaoWfInfo {
    #[serde(rename = "name")]
    pub name: Option<String>,
    #[serde(rename = "value")]
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YoudaoWebItem {
    #[serde(rename = "key")]
    pub key: String,
    #[serde(rename = "value")]
    pub value: Vec<String>,
}

/// 简化的单词数据结构，用于下载
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordEntry {
    pub word: String,
    pub phonetic: Option<String>,
    pub definition: String,
    pub part_of_speech: Option<String>,
}

impl WordEntry {
    pub fn from_youdao(response: &YoudaoResponse) -> Vec<Self> {
        let mut entries = Vec::new();

        // 从 basic 中获取主要释义
        if let Some(basic) = &response.basic {
            // 音标
            let phonetic = basic.us_phonetic.clone()
                .or_else(|| basic.uk_phonetic.clone())
                .or_else(|| basic.phonetic.clone());

            // 基本释义
            if let Some(translations) = &basic.translations {
                for trans in translations {
                    if let (Some(word_name), Some(defs)) = (&trans.word, &trans.trans) {
                        let definition = defs.join("；");
                        entries.push(WordEntry {
                            word: word_name.clone(),
                            phonetic: phonetic.clone(),
                            definition: definition.clone(),
                            part_of_speech: None,
                        });
                    }
                }
            }
        }

        // 如果 basic 没有翻译，使用 translation 字段
        if entries.is_empty() {
            if let Some(translations) = &response.translation {
                for trans in translations {
                    entries.push(WordEntry {
                        word: response.query.clone(),
                        phonetic: None,
                        definition: trans.clone(),
                        part_of_speech: None,
                    });
                }
            }
        }

        entries
    }
}

/// 生成有道API签名
fn generate_sign(app_key: &str, q: &str, salt: &str, curtime: &str, app_secret: &str) -> String {
    let sign_str = format!("{}{}{}{}{}", app_key, q, salt, curtime, app_secret);
    format!("{:x}", md5::compute(sign_str.as_bytes()))
}

/// 查询单词（同步版本）
pub fn lookup_word_sync(config: &YoudaoConfig, word: &str) -> Result<YoudaoResponse, String> {
    if config.app_key.is_empty() || config.app_secret.is_empty() {
        return Err("API配置未设置，请先在设置中配置有道API".to_string());
    }

    let salt = uuid::Uuid::new_v4().to_string();
    let curtime = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()) as i64;
    let curtime_str = curtime.to_string();

    let sign = generate_sign(&config.app_key, word, &salt, &curtime_str, &config.app_secret);

    let params = [
        ("q", word),
        ("from", "en"),
        ("to", "zh"),
        ("appKey", &config.app_key),
        ("salt", &salt),
        ("sign", &sign),
        ("curtime", &curtime_str),
        ("signType", "v3"),
        ("dict", "true"),
    ];

    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(&config.api_url)
        .form(&params)
        .send()
        .map_err(|e| format!("网络请求失败: {}", e))?;

    let result: YoudaoResponse = resp
        .json()
        .map_err(|e| format!("解析响应失败: {}", e))?;

    if result.error_code != "0" {
        return Err(format!("API错误: {}", result.error_code));
    }

    Ok(result)
}

/// 查询单词（异步版本）
pub async fn lookup_word(config: &YoudaoConfig, word: &str) -> Result<YoudaoResponse, String> {
    if config.app_key.is_empty() || config.app_secret.is_empty() {
        return Err("API配置未设置，请先在设置中配置有道API".to_string());
    }

    let salt = uuid::Uuid::new_v4().to_string();
    let curtime = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()) as i64;
    let curtime_str = curtime.to_string();

    let sign = generate_sign(&config.app_key, word, &salt, &curtime_str, &config.app_secret);

    let mut params = HashMap::new();
    params.insert("q", word);
    params.insert("from", "en");
    params.insert("to", "zh");
    params.insert("appKey", &config.app_key);
    params.insert("salt", &salt);
    params.insert("sign", &sign);
    params.insert("curtime", &curtime_str);
    params.insert("signType", "v3");
    params.insert("dict", "true");

    let client = reqwest::Client::new();
    let resp = client
        .post(&config.api_url)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("网络请求失败: {}", e))?;

    let result: YoudaoResponse = resp
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    if result.error_code != "0" {
        return Err(format!("API错误: {}", result.error_code));
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_generation() {
        // 测试签名生成
        let sign = generate_sign("app_key", "hello", "salt", "1234567890", "app_secret");
        assert_eq!(sign.len(), 32);
    }

    #[test]
    fn test_word_entry_from_response() {
        let response = YoudaoResponse {
            query: "hello".to_string(),
            error_code: "0".to_string(),
            translation: Some(vec!["你好".to_string()]),
            basic: Some(YoudaoBasic {
                phonetic: Some("/həˈloʊ/".to_string()),
                us_phonetic: Some("/həˈloʊ/".to_string()),
                uk_phonetic: Some("/həˈləʊ/".to_string()),
                wfs: None,
                exam_type: None,
                translations: None,
            }),
            web: None,
        };

        let entries = WordEntry::from_youdao(&response);
        assert!(!entries.is_empty());
    }
}