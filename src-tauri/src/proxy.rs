use reqwest::Client;
use serde_json::Value;

pub struct PythonProxy {
    client: Client,
    base_url: String,
}

impl PythonProxy {
    pub fn new(port: u16) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .unwrap(),
            base_url: format!("http://127.0.0.1:{}", port),
        }
    }

    pub async fn post(&self, path: &str, body: Value) -> Result<Value, String> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("请求失败: {}", e))?;

        resp.json::<Value>()
            .await
            .map_err(|e| format!("解析响应失败: {}", e))
    }

    pub async fn get(&self, path: &str) -> Result<Value, String> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("请求失败: {}", e))?;

        resp.json::<Value>()
            .await
            .map_err(|e| format!("解析响应失败: {}", e))
    }

    pub async fn health_check(&self) -> bool {
        let url = format!("{}/api/health", self.base_url);
        match self.client.get(&url).send().await {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }
}
