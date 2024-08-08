use std::sync::mpsc;

use super::utils::parser::{parse_atoms_from_pdb_file, ParseError, ParsedMolecule};

pub enum FileResponse {
    FileParsed(ParsedMolecule),
    ParsingFailed(ParseError),
    NoContent,
}

enum AsyncFileMessage {
    FileLoaded(Vec<u8>),
}

pub struct AsyncFileLoader {
    channel: (
        mpsc::Sender<AsyncFileMessage>,
        mpsc::Receiver<AsyncFileMessage>,
    ),
}

impl Default for AsyncFileLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// TODO: docs, explain why it's async
impl AsyncFileLoader {
    pub fn new() -> Self {
        Self {
            channel: mpsc::channel(),
        }
    }

    pub fn open_file_dialog(&self) {
        let dispatch = self.channel.0.clone();
        execute(async move {
            let file_dialog = rfd::AsyncFileDialog::new().add_filter("PDB", &["pdb", "cif"]);

            if let Some(files) = file_dialog.pick_files().await {
                for file in files {
                    let content = file.read().await;
                    dispatch.send(AsyncFileMessage::FileLoaded(content)).ok();
                }
            }
        })
    }

    pub fn get_parsed_files(&mut self) -> FileResponse {
        let Ok(AsyncFileMessage::FileLoaded(content)) = self.channel.1.try_recv() else {
            return FileResponse::NoContent;
        };

        match parse_atoms_from_pdb_file(&content) {
            Ok(parsed) => FileResponse::FileParsed(parsed),
            Err(err) => FileResponse::ParsingFailed(err),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn execute<F: futures::Future<Output = ()> + Send + 'static>(f: F) {
    std::thread::spawn(move || futures::executor::block_on(f));
}

#[cfg(target_arch = "wasm32")]
fn execute<F: futures::Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}
