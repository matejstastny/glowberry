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

    pub async fn search_modpacks(
        &self,
        query: &str,
        limit: u32,
        offset: u32,
    ) -> Result<SearchResult, GlowberryError> {
        let facets = r#"[["project_type:modpack"]]"#;
        let resp = self
            .client
            .get(format!("{BASE_URL}/search"))
            .query(&[
                ("query", query),
                ("limit", &limit.to_string()),
                ("offset", &offset.to_string()),
                ("facets", facets),
            ])
            .send()
            .await?
            .error_for_status()?
            .json::<SearchResult>()
            .await?;
        Ok(resp)
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

    pub async fn list_versions(&self, project_id: &str) -> Result<Vec<Version>, GlowberryError> {
        let resp = self
            .client
            .get(format!("{BASE_URL}/project/{project_id}/version"))
            .send()
            .await?
            .error_for_status()?
            .json::<Vec<Version>>()
            .await?;
        Ok(resp)
    }

    pub async fn get_version(&self, version_id: &str) -> Result<Version, GlowberryError> {
        let resp = self
            .client
            .get(format!("{BASE_URL}/version/{version_id}"))
            .send()
            .await?
            .error_for_status()?
            .json::<Version>()
            .await?;
        Ok(resp)
    }
}
