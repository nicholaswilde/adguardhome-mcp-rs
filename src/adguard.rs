use crate::config::AppConfig;
use crate::error::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct AdGuardClient {
    pub client: reqwest::Client,
    pub config: AppConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Status {
    pub version: String,
    pub language: String,
    pub protection_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DnsRewrite {
    pub domain: String,
    pub answer: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Stats {
    pub num_dns_queries: u64,
    pub num_blocked_filtering: u64,
    pub num_replaced_safebrowsing: u64,
    pub num_replaced_safesearch: u64,
    pub num_replaced_parental: u64,
    pub avg_processing_time: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryLogEntry {
    pub client: String,
    pub elapsed_ms: String,
    pub reason: String,
    pub status: String,
    pub time: String,
    pub question: QueryLogQuestion,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryLogQuestion {
    pub name: String,
    #[serde(rename = "type")]
    pub qtype: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryLogResponse {
    pub data: Vec<QueryLogEntry>,
}

impl AdGuardClient {
    pub fn new(config: AppConfig) -> Self {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(config.no_verify_ssl)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { client, config }
    }

    pub async fn get_status(&self) -> Result<Status> {
        let url = format!(
            "http://{}:{}/control/status",
            self.config.adguard_host, self.config.adguard_port
        );
        let mut request = self.client.get(&url);

        if let (Some(user), Some(pass)) =
            (&self.config.adguard_username, &self.config.adguard_password)
        {
            request = request.basic_auth(user, Some(pass));
        }

        let response = request.send().await?.error_for_status()?;
        let status = response.json::<Status>().await?;
        Ok(status)
    }

    pub async fn get_stats(&self, time_period: Option<&str>) -> Result<Stats> {
        let mut url = format!(
            "http://{}:{}/control/stats",
            self.config.adguard_host, self.config.adguard_port
        );
        if let Some(period) = time_period {
            url.push_str(&format!("?time_period={}", period));
        }
        let mut request = self.client.get(&url);

        if let (Some(user), Some(pass)) =
            (&self.config.adguard_username, &self.config.adguard_password)
        {
            request = request.basic_auth(user, Some(pass));
        }

        let response = request.send().await?.error_for_status()?;
        let stats = response.json::<Stats>().await?;
        Ok(stats)
    }

    pub async fn get_query_log(
        &self,
        search: Option<&str>,
        filter: Option<&str>,
        limit: Option<u32>,
    ) -> Result<QueryLogResponse> {
        let mut url = format!(
            "http://{}:{}/control/querylog",
            self.config.adguard_host, self.config.adguard_port
        );
        let mut params = Vec::new();
        if let Some(s) = search {
            params.push(format!("search={}", s));
        }
        if let Some(f) = filter {
            params.push(format!("filter={}", f));
        }
        if let Some(l) = limit {
            params.push(format!("limit={}", l));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        let mut request = self.client.get(&url);

        if let (Some(user), Some(pass)) =
            (&self.config.adguard_username, &self.config.adguard_password)
        {
            request = request.basic_auth(user, Some(pass));
        }

        let response = request.send().await?.error_for_status()?;
        let log = response.json::<QueryLogResponse>().await?;
        Ok(log)
    }

    pub async fn list_rewrites(&self) -> Result<Vec<DnsRewrite>> {
        let url = format!(
            "http://{}:{}/control/rewrite/list",
            self.config.adguard_host, self.config.adguard_port
        );
        let mut request = self.client.get(&url);

        if let (Some(user), Some(pass)) =
            (&self.config.adguard_username, &self.config.adguard_password)
        {
            request = request.basic_auth(user, Some(pass));
        }

        let response = request.send().await?.error_for_status()?;
        let rewrites = response.json::<Vec<DnsRewrite>>().await?;
        Ok(rewrites)
    }

    pub async fn add_rewrite(&self, rewrite: DnsRewrite) -> Result<()> {
        let url = format!(
            "http://{}:{}/control/rewrite/add",
            self.config.adguard_host, self.config.adguard_port
        );
        let mut request = self.client.post(&url).json(&rewrite);

        if let (Some(user), Some(pass)) =
            (&self.config.adguard_username, &self.config.adguard_password)
        {
            request = request.basic_auth(user, Some(pass));
        }

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn delete_rewrite(&self, rewrite: DnsRewrite) -> Result<()> {
        let url = format!(
            "http://{}:{}/control/rewrite/delete",
            self.config.adguard_host, self.config.adguard_port
        );
        let mut request = self.client.post(&url).json(&rewrite);

        if let (Some(user), Some(pass)) =
            (&self.config.adguard_username, &self.config.adguard_password)
        {
            request = request.basic_auth(user, Some(pass));
        }

        request.send().await?.error_for_status()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn test_config(host: String, port: u16) -> AppConfig {
        AppConfig {
            adguard_host: host,
            adguard_port: port,
            adguard_username: None,
            adguard_password: None,
            mcp_transport: "stdio".to_string(),
            lazy_mode: false,
            http_port: 3000,
            http_auth_token: None,
            log_level: "info".to_string(),
            no_verify_ssl: true,
        }
    }

    #[tokio::test]
    async fn test_list_rewrites() {
        let server = MockServer::start().await;
        let config = test_config(
            server
                .uri()
                .replace("http://", "")
                .split(':')
                .next()
                .unwrap()
                .to_string(),
            server
                .uri()
                .split(':')
                .next_back()
                .unwrap()
                .parse()
                .unwrap(),
        );
        let client = AdGuardClient::new(config);

        Mock::given(method("GET"))
            .and(path("/control/rewrite/list"))
            .respond_with(ResponseTemplate::new(200).set_body_json(vec![DnsRewrite {
                domain: "example.com".to_string(),
                answer: "1.2.3.4".to_string(),
            }]))
            .mount(&server)
            .await;

        let rewrites = client.list_rewrites().await.unwrap();
        assert_eq!(rewrites.len(), 1);
        assert_eq!(rewrites[0].domain, "example.com");
    }

    #[tokio::test]
    async fn test_add_rewrite() {
        let server = MockServer::start().await;
        let config = test_config(
            server
                .uri()
                .replace("http://", "")
                .split(':')
                .next()
                .unwrap()
                .to_string(),
            server
                .uri()
                .split(':')
                .next_back()
                .unwrap()
                .parse()
                .unwrap(),
        );
        let client = AdGuardClient::new(config);

        Mock::given(method("POST"))
            .and(path("/control/rewrite/add"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        let rewrite = DnsRewrite {
            domain: "example.com".to_string(),
            answer: "1.2.3.4".to_string(),
        };
        client.add_rewrite(rewrite).await.unwrap();
    }

    #[tokio::test]
    async fn test_delete_rewrite() {
        let server = MockServer::start().await;
        let config = test_config(
            server
                .uri()
                .replace("http://", "")
                .split(':')
                .next()
                .unwrap()
                .to_string(),
            server
                .uri()
                .split(':')
                .next_back()
                .unwrap()
                .parse()
                .unwrap(),
        );
        let client = AdGuardClient::new(config);

        Mock::given(method("POST"))
            .and(path("/control/rewrite/delete"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        let rewrite = DnsRewrite {
            domain: "example.com".to_string(),
            answer: "1.2.3.4".to_string(),
        };
        client.delete_rewrite(rewrite).await.unwrap();
    }

    #[tokio::test]
    async fn test_get_stats() {
        let server = MockServer::start().await;
        let config = test_config(
            server
                .uri()
                .replace("http://", "")
                .split(':')
                .next()
                .unwrap()
                .to_string(),
            server
                .uri()
                .split(':')
                .next_back()
                .unwrap()
                .parse()
                .unwrap(),
        );
        let client = AdGuardClient::new(config);

        Mock::given(method("GET"))
            .and(path("/control/stats"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "num_dns_queries": 100,
                "num_blocked_filtering": 10,
                "num_replaced_safebrowsing": 5,
                "num_replaced_safesearch": 2,
                "num_replaced_parental": 1,
                "avg_processing_time": 0.05
            })))
            .mount(&server)
            .await;

        let stats = client.get_stats(None).await.unwrap();
        assert_eq!(stats.num_dns_queries, 100);
        assert_eq!(stats.num_blocked_filtering, 10);
    }

    #[tokio::test]
    async fn test_get_query_log() {
        let server = MockServer::start().await;
        let config = test_config(
            server
                .uri()
                .replace("http://", "")
                .split(':')
                .next()
                .unwrap()
                .to_string(),
            server
                .uri()
                .split(':')
                .next_back()
                .unwrap()
                .parse()
                .unwrap(),
        );
        let client = AdGuardClient::new(config);

        Mock::given(method("GET"))
            .and(path("/control/querylog"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [
                    {
                        "client": "127.0.0.1",
                        "elapsed_ms": "0.1",
                        "reason": "NotFilteredNotFound",
                        "status": "NOERROR",
                        "time": "2021-01-01T00:00:00Z",
                        "question": {
                            "name": "google.com",
                            "type": "A"
                        }
                    }
                ]
            })))
            .mount(&server)
            .await;

        let log = client.get_query_log(None, None, None).await.unwrap();
        assert_eq!(log.data.len(), 1);
        assert_eq!(log.data[0].question.name, "google.com");
    }
}
