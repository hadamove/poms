use crate::parser::parse::{parse_files, ParsedFile};
use anyhow::Error;
use std::sync::mpsc::{self, Receiver, Sender};

pub enum FileResponse {
    ParsedFiles(Vec<ParsedFile>),
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
        let (sender, receiver) = mpsc::channel();
        Self {
            channel: (sender, receiver),
        }
    }

    pub fn load_pdb_files(&self) {
        let dispatch = self.channel.0.clone();
        execute(async move {
            let file_dialog = rfd::AsyncFileDialog::new().add_filter("PDB", &["pdb"]);

            if let Some(files) = file_dialog.pick_files().await {
                let mut contents = Vec::new();
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
