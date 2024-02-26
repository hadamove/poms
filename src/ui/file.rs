use crate::parser::parse::{parse_files, ParsedMolecule};
use anyhow::Error;
use std::sync::mpsc::{self, Receiver, Sender};

pub enum FileResponse {
    ParsedFiles(Vec<ParsedMolecule>),
    ParsingFailed(Error),
    NoContent,
}

enum AsyncFileMessage {
    FilesLoaded(Vec<Vec<u8>>),
}

pub struct AsyncFileLoader {
    channel: (Sender<AsyncFileMessage>, Receiver<AsyncFileMessage>),
}

impl AsyncFileLoader {
    pub fn new() -> Self {
        Self {
            channel: mpsc::channel(),
        }
    }

    /// How this could be rewritten to be more performant with multiple files:
    /// - a constant `MAX_BUFFERED_FILES: usize = 10` is defined
    /// - a `Vec<Vec<u8>>` is used to store the files
    /// - the for loop reading files halts when the `Vec` reaches `MAX_BUFFERED_FILES`
    /// - a message is sent to the main thread with each file read
    /// - two directional buffering - some files are buffered before "current" frame and some after
    pub fn load_pdb_files(&self) {
        let dispatch = self.channel.0.clone();
        execute(async move {
            let file_dialog = rfd::AsyncFileDialog::new().add_filter("PDB", &["pdb", "cif"]);

            if let Some(files) = file_dialog.pick_files().await {
                let mut contents = Vec::new();
                // TODO: Replace this with stream
                for file in files {
                    contents.push(file.read().await);
                }
                dispatch.send(AsyncFileMessage::FilesLoaded(contents)).ok();
            }
        })
    }

    pub fn get_parsed_files(&mut self) -> FileResponse {
        if let Ok(AsyncFileMessage::FilesLoaded(contents)) = self.channel.1.try_recv() {
            match parse_files(contents) {
                Ok(parsed) => FileResponse::ParsedFiles(parsed),
                Err(err) => FileResponse::ParsingFailed(err),
            }
        } else {
            FileResponse::NoContent
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
