use super::Assembly;

pub struct DownloadApiResponse {
    pub raw_data: Vec<u8>,
}

#[derive(Clone, Default)]
pub struct PdbDownloadApi {
    client: reqwest::Client,
}

impl PdbDownloadApi {
    pub async fn download_assembly(
        &self,
        assembly: &Assembly,
    ) -> anyhow::Result<DownloadApiResponse> {
        let url = Self::forge_url(assembly);

        let response = self
            .client
            .get(&url)
            .header("accept", "text/plain")
            .send()
            .await
            .map_err(anyhow::Error::new)?;

        let raw_data = response.bytes().await.map_err(anyhow::Error::new)?.to_vec();

        Ok(DownloadApiResponse { raw_data })
    }

    fn forge_url(assembly: &Assembly) -> String {
        format!(
            "https://models.rcsb.org/v1/{}/assembly?name={}&encoding=cif&copy_all_categories=false&download=true",
            assembly.pdb_id,
            assembly.assembly_id
        )
    }
}
