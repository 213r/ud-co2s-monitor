use super::UDCO2SDATA;

pub struct InfluxDBExporter {
    url: String,
    token: String,
}

#[derive(Debug, thiserror::Error)]
pub enum InfluxDBError {
    #[error("HTTPリクエスト失敗: {0}")]
    Request(#[from] reqwest::Error),

    #[error("InfluxDBのエラー応答: status = {status}, body = {body}")]
    InfluxDBResponse {
        status: reqwest::StatusCode,
        body: String,
    },
}

impl InfluxDBExporter {
    pub fn new(org: &str, bucket: &str, token: &str) -> Self {
        let url = format!(
            "http://localhost:8086/api/v2/write?org={}&bucket={}&precision=s",
            org, bucket
        );
        InfluxDBExporter {
            url,
            token: token.to_string(),
        }
    }

    pub async fn set(&mut self, data: &UDCO2SDATA) -> Result<(), InfluxDBError> {
        let timestamp = chrono::Utc::now().timestamp();
        let line = format!(
            "home,device='udco2s' temperature={},humidity={},co2={} {}",
            data.temp, data.hum, data.co2, timestamp
        );

        let client = reqwest::Client::new();
        let res = client
            .post(&self.url)
            .header("Authorization", format!("Token {}", self.token))
            .header("Content-Type", "text/plain")
            .body(line)
            .send()
            .await?;

        if !res.status().is_success() {
            let status = res.status();
            let body = res.text().await?;
            return Err(InfluxDBError::InfluxDBResponse { status, body });
        }
        Ok(())
    }
}
