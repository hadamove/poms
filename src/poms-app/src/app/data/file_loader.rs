use std::sync::mpsc;

use super::molecule_parser::{parse_multiple_files, ParsedMolecule};
use super::pdb_apis::{download_api::PdbDownloadApi, search_api::PdbSearchApi};
use super::{Assembly, RawFile};

pub(crate) enum DownloadProgress {
    Downloading { bytes_downloaded: usize },
    Parsing,
    Finished,
}

pub(crate) enum AsyncWorkResult {
    FilesParsed {
        result: anyhow::Result<Vec<ParsedMolecule>>,
    },
    SearchResultsParsed {
        result: anyhow::Result<Vec<Assembly>>,
    },
    DownloadProgressed {
        progress: DownloadProgress,
    },
}

/// Asynchronously loads and downloads files. This design ensures compatibility across
/// different platforms, including the web where blocking the main thread is prohibited.
/// Also blocking the main thread would cause the UI to freeze, which is bad UX.
pub(crate) struct FileLoader {
    data_channel: (
        mpsc::Sender<AsyncWorkResult>,
        mpsc::Receiver<AsyncWorkResult>,
    ),
    download_api: PdbDownloadApi,
    search_api: PdbSearchApi,
}

impl FileLoader {
    /// Creates a new instance of `FileLoader`.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Opens an async file dialog for selecting files without blocking the main thread.
    ///
    /// The selected files are read and sent over a channel for processing.
    /// User may select multiple files, which are interpreted not as separate molecules but as frames (animation) of a single molecule.
    pub(crate) fn pick_files(&self) {
        let dispatch = self.data_channel.0.clone();
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

                let parsed_files = parse_multiple_files(loaded_files);

                dispatch
                    .send(AsyncWorkResult::FilesParsed {
                        result: parsed_files,
                    })
                    .ok();
            }
        })
    }

    /// Downloads a file on an asynchronous basis using the provided `Assembly` object used to identify the file to download.
    /// Uses a minimal wrapper around the RCSB's public API to fetch the file content.
    /// Fetched files are returned to the main thread via a channel.
    pub(crate) fn download_file(&self, assembly: Assembly) {
        let dispatch = self.data_channel.0.clone();
        let download_api = self.download_api.clone();
        execute(async move {
            let _ = download_api.download_assembly(&assembly, dispatch).await;
        });
    }

    /// Initializes an asychronous task that does full-text search for PDB files using the RCSB's public API.
    /// Fetched results are returned to the main thread via a channel.
    pub(crate) fn search_pdb_files(&self, query: String) {
        let dispatch = self.data_channel.0.clone();
        let search_api = self.search_api.clone();
        execute(async move {
            let _ = search_api.fulltext_search_debounced(&query, dispatch).await;
        });
    }

    /// Drains the results of all asynchronous tasks
    pub(crate) fn collect_data_events(&mut self) -> Vec<AsyncWorkResult> {
        std::iter::from_fn(|| self.data_channel.1.try_recv().ok()).collect()
    }
}

impl Default for FileLoader {
    fn default() -> Self {
        Self {
            data_channel: mpsc::channel(),
            download_api: PdbDownloadApi::default(),
            search_api: PdbSearchApi::default(),
        }
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
