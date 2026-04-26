use crate::error::GlowberryError;
use crate::modrinth::types::*;

const BASE_URL: &str = "https://api.modrinth.com/v2";

pub struct ModrinthApi {
    client: reqwest::Client,
}

impl ModrinthApi {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }

    pub async fn get_project(&self, id_or_slug: &str) -> Result<Project, GlowberryError> {
        let resp = self
            .client
            .get(format!("{BASE_URL}/project/{id_or_slug}"))
            .send()
            .await?
            .error_for_status()?
            .json::<Project>()
            .await?;
        Ok(resp)
    }
}
