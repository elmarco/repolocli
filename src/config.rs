use url::Url;

#[derive(Debug, Serialize, Deserialize)]
pub struct Configuration {
    #[serde(with = "url_serde")]
    #[serde(rename = "repology_url")]
    repology_url: Url,

    #[serde(rename = "allowlist")]
    allowlist: Vec<String>,

    #[serde(rename = "denylist")]
    denylist: Vec<String>,
}

impl Configuration {
    pub fn repology_url(&self) -> &Url {
        &self.repology_url
    }

    pub fn allowlist(&self) -> &Vec<String> {
        &self.allowlist
    }

    pub fn denylist(&self) -> &Vec<String> {
        &self.denylist
    }

}

