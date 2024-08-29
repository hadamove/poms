use futures::StreamExt;
use std::sync::mpsc;

use super::Assembly;
use crate::app::data::file_loader::{AsyncWorkResult, DownloadProgress, RawFile};
use crate::app::data::molecule_parser::parse_multiple_files;

#[derive(Clone, Default)]
pub struct PdbDownloadApi {
    client: reqwest::Client,
}

impl PdbDownloadApi {
    pub async fn download_assembly(
        &self,
        assembly: &Assembly,
        dispatch: mpsc::Sender<AsyncWorkResult>,
    ) -> anyhow::Result<()> {
        let url = Self::forge_url(assembly);

        let response = self
            .client
            .get(&url)
            .header("accept", "application/octet-stream")
            .send()
            .await
            .map_err(anyhow::Error::new)?;

        let mut data = Vec::new();
        let mut bytes_downloaded = 0usize;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            Self::report_progress(
                &dispatch,
                DownloadProgress::Downloading { bytes_downloaded },
            );

            let chunk = chunk.map_err(anyhow::Error::new)?;
            data.extend_from_slice(&chunk);
            bytes_downloaded += chunk.len();
        }

        Self::report_progress(&dispatch, DownloadProgress::Parsing);

        let raw_file = RawFile {
            name: assembly.to_string(),
            content: data,
        };

        let parsed = parse_multiple_files(vec![raw_file]);

        Self::report_progress(&dispatch, DownloadProgress::Finished);

        dispatch
            .send(AsyncWorkResult::FilesParsed { result: parsed })
            .ok();

        Ok(())
    }

    fn forge_url(assembly: &Assembly) -> String {
        format!(
            "https://models.rcsb.org/v1/{}/assembly?name={}&encoding=cif",
            assembly.pdb_id, assembly.assembly_id
        )
    }

    fn report_progress(dispatch: &mpsc::Sender<AsyncWorkResult>, progress: DownloadProgress) {
        dispatch
            .send(AsyncWorkResult::DownloadProgressed { progress })
            .ok();
    }
}
