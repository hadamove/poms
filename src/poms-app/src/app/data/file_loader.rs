use std::{str::FromStr, sync::mpsc};

use super::{
    molecule_parser::{parse_atoms_from_pdb_file, ParsedMolecule},
    pdb_apis::{download_api::PdbDownloadApi, search_api::PdbSearchApi, Assembly},
};

/// Represents the possible outcomes when handling files.
pub enum DataEvent {
    FilesParsed {
        result: anyhow::Result<Vec<ParsedMolecule>>,
    },
    SearchResultsParsed {
        result: anyhow::Result<Vec<Assembly>>,
    },
}

pub enum AsyncWorkResult {
    FilesReceived { files: Vec<RawFile> },
    SearchResultsReceived { results: Vec<String> },
}

/// Holds the raw content of a loaded file.
pub struct RawFile {
    pub name: String,
    pub content: Vec<u8>,
}

/// Asynchronously loads and downloads files. This design ensures compatibility across
/// different platforms, including the web where blocking the main thread is prohibited.
/// Also blocking the main thread would cause the UI to freeze, which is bad UX.
pub struct FileLoader {
    channel: (
        mpsc::Sender<AsyncWorkResult>,
        mpsc::Receiver<AsyncWorkResult>,
    ),
}

impl FileLoader {
    /// Creates a new instance of `FileLoader`.
    pub fn new() -> Self {
        Self {
            channel: mpsc::channel(),
        }
    }

    /// Opens an async file dialog for selecting files without blocking the main thread.
    ///
    /// The selected files are read and sent over a channel for processing.
    /// User may select multiple files, which are interpreted not as separate molecules but as frames (animation) of a single molecule.
    pub fn pick_files(&self) {
        let dispatch = self.channel.0.clone();
        execute(async move {
            let file_dialog = rfd::AsyncFileDialog::new().add_filter("PDB", &["pdb", "cif"]);
            if let Some(files) = file_dialog.pick_files().await {
                // Load all files asynchronously
                let loaded_files: Vec<RawFile> =
                    futures::future::join_all(files.iter().map(|file| async {
                        RawFile {
                            name: file.file_name(),
                            content: file.read().await,
                        }
                    }))
                    .await;

                dispatch
                    .send(AsyncWorkResult::FilesReceived {
                        files: loaded_files,
                    })
                    .ok();
            }
        })
    }

    /// Downloads a file on an asynchronous basis using the provided `Assembly` object used to identify the file to download.
    /// Uses a minimal wrapper around the RCSB's public API to fetch the file content.
    /// Fetched files are returned to the main thread via a channel.
    pub fn download_file(&self, assembly: Assembly) {
        let dispatch = self.channel.0.clone();
        execute(async move {
            let download_api = PdbDownloadApi::new();
            if let Ok(response) = download_api.download_assembly(&assembly).await {
                dispatch
                    .send(AsyncWorkResult::FilesReceived {
                        files: vec![RawFile {
                            name: assembly.to_string(),
                            content: response.raw_data,
                        }],
                    })
                    .ok();
            }
        });
    }

    /// Initializes an asychronous task that does full-text search for PDB files using the RCSB's public API.
    /// Fetched results are returned to the main thread via a channel.
    pub fn search_pdb_files(&self, query: String) {
        if query.is_empty() {
            return;
        }

        let dispatch = self.channel.0.clone();
        execute(async move {
            let search_api = PdbSearchApi::new();
            if let Ok(response) = search_api.fulltext_search(&query).await {
                dispatch
                    .send(AsyncWorkResult::SearchResultsReceived {
                        results: response.result_set,
                    })
                    .ok();
            }
        });
    }

    /// Retrieves and parses files selected via the file dialog.
    ///
    /// Checks for received files, then attempts to parse them, returns a `FileResponse` indicating the result.
    pub fn try_process_single_event(&mut self) -> Option<DataEvent> {
        self.channel.1.try_recv().ok().map(|message| match message {
            AsyncWorkResult::FilesReceived { files } => DataEvent::FilesParsed {
                result: Self::parse_files(files),
            },
            AsyncWorkResult::SearchResultsReceived { results } => DataEvent::SearchResultsParsed {
                result: Self::parse_search_results(results),
            },
        })
    }

    fn parse_search_results(results: Vec<String>) -> anyhow::Result<Vec<Assembly>> {
        results
            .into_iter()
            .map(|result_str| Assembly::from_str(&result_str).map_err(anyhow::Error::msg))
            .collect::<anyhow::Result<Vec<Assembly>>>()
    }

    fn parse_files(loaded_files: Vec<RawFile>) -> anyhow::Result<Vec<ParsedMolecule>> {
        loaded_files
            .into_iter()
            .map(parse_atoms_from_pdb_file)
            .collect::<anyhow::Result<Vec<ParsedMolecule>>>()
    }
}

impl Default for FileLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Executes a future on a separate thread or context to avoid blocking the main thread.
///
/// Uses `std::thread::spawn` for native platforms and `wasm_bindgen_futures::spawn_local`
/// for web platforms, ensuring cross-platform compatibility.
#[cfg(not(target_arch = "wasm32"))]
fn execute<F: futures::Future<Output = ()> + Send + 'static>(f: F) {
    // Create a Tokio runtime manually
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(f); // Use Tokio's block_on to run the future
    });
}

#[cfg(target_arch = "wasm32")]
fn execute<F: futures::Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}
