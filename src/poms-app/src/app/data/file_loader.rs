use std::sync::mpsc;

use super::molecule_parser::{parse_atoms_from_pdb_file, ParseError, ParsedMolecule};

/// Represents the possible outcomes when handling files.
pub enum FileResponse {
    FileParsed { molecule: ParsedMolecule },
    ParsingFailed { error: ParseError },
    NoContent,
}

/// Holds the raw content of a loaded file.
struct FileData {
    content: Vec<u8>,
}

/// Asynchronously loads and parses files, ensuring the main thread remains unblocked.
///
/// Once a file is selected, it is read and sent over a channel to be parsed by the main
/// application thread. This design ensures compatibility across different platforms,
/// including the web where blocking the main thread is prohibited.
pub struct FileLoader {
    channel: (mpsc::Sender<FileData>, mpsc::Receiver<FileData>),
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
    pub fn open_file_dialog(&self) {
        let dispatch = self.channel.0.clone();
        execute(async move {
            let file_dialog = rfd::AsyncFileDialog::new().add_filter("PDB", &["pdb", "cif"]);
            if let Some(files) = file_dialog.pick_files().await {
                for file in files {
                    let content = file.read().await;
                    dispatch.send(FileData { content }).ok();
                }
            }
        })
    }

    /// Retrieves and parses files selected via the file dialog.
    ///
    /// Checks for received file data, then attempts to parse it into a `ParsedMolecule`.
    /// Returns a `FileResponse` indicating the result.
    pub fn get_parsed_files(&mut self) -> FileResponse {
        if let Ok(FileData { content }) = self.channel.1.try_recv() {
            match parse_atoms_from_pdb_file(&content) {
                Ok(molecule) => FileResponse::FileParsed { molecule },
                Err(error) => FileResponse::ParsingFailed { error },
            }
        } else {
            FileResponse::NoContent
        }
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
    std::thread::spawn(move || futures::executor::block_on(f));
}

#[cfg(target_arch = "wasm32")]
fn execute<F: futures::Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}
