use thiserror::Error;

#[derive(Error, Debug)]
pub enum NotifierError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("Notification service error: {0}")]
    ServiceError(String),
}

pub struct NotifierService {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
}

impl NotifierService {
    pub fn new(base_url: String, api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
            api_key,
        }
    }

    pub async fn send_password_reset(
        &self,
        to: &str,
        user_name: &str,
        action_url: &str,
    ) -> Result<(), NotifierError> {
        let payload = serde_json::json!({
            "to": to,
            "channel": "email",
            "template": "password_reset",
            "data": {
                "user_name": user_name,
                "action_url": action_url
            }
        });

        let response = self
            .client
            .post(format!("{}/api/v1/send", self.base_url))
            .header("x-notifier-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            tracing::error!("Notifier returned error: {} - {}", status, body);
            Err(NotifierError::ServiceError(format!(
                "HTTP {}: {}",
                status, body
            )))
        }
    }

    pub async fn send_welcome(&self, to: &str, user_name: &str) -> Result<(), NotifierError> {
        let payload = serde_json::json!({
            "to": to,
            "channel": "email",
            "template": "welcome",
            "data": {
                "user_name": user_name
            }
        });

        let response = self
            .client
            .post(format!("{}/api/v1/send", self.base_url))
            .header("x-notifier-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            tracing::error!("Notifier returned error: {} - {}", status, body);
            Err(NotifierError::ServiceError(format!(
                "HTTP {}: {}",
                status, body
            )))
        }
    }
}

impl Clone for NotifierService {
    fn clone(&self) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: self.base_url.clone(),
            api_key: self.api_key.clone(),
        }
    }
}