use std::{ffi::OsStr, fs, path::{Path, PathBuf}};

use bevy_egui::egui::Ui;

pub enum Library {
    /// A folder whose contents have not yet been read.
    UnloadedFolder { path: PathBuf, name: String },

    /// A folder whose contents have been read.
    LoadedFolder {
        path: PathBuf,
        name: String,
        contents: Vec<Library>,
    },

    /// A file that can be loaded into Miratope.
    File { path: PathBuf, name: String },
}

/// Reads a folder's name from the `.metadata` file, or defaults to the folder's
/// actual name.
fn get_name(path: &Path) -> Result<String, &str> {
    assert!(path.is_dir(), "Path {:?} not a directory!", path);

    let new_path = path.join(".metadata");

    if path.exists() {
        String::from_utf8(fs::read(new_path).map_err(|_| "File could not be read.")?)
            .map_err(|_| "File not UTF-8.")
    } else {
        Ok(String::from(
            path.file_name().map(|f| f.to_str()).flatten().unwrap_or(""),
        ))
    }
}

impl Library {
    /// An unloaded folder.
    pub fn new(path: &impl AsRef<OsStr>) -> Self {
        let path = PathBuf::from(&path);
        let name = get_name(&path).unwrap();

        Self::UnloadedFolder { path, name }
    }

    /// Shows the library.
    pub fn show(&mut self, ui: &mut Ui) -> Option<PathBuf> {
        match self {
            // Shows a collapsing drop-down, and loads the folder in case it's clicked.
            Self::UnloadedFolder { path, name } => {
                // Clones so that the closure doesn't require unique access.
                let path = path.clone();
                let name = name.clone();

                let mut res = None;

                ui.collapsing(name.clone(), |ui| {
                    let mut contents = Vec::new();

                    // Reads through the entries of the folders.
                    match fs::read_dir(path.clone()) {
                        Ok(dir_entry) => {
                            // For every entry in the folder:
                            for entry in dir_entry {
                                match entry {
                                    Ok(entry) => {
                                        let path = entry.path();

                                        // Adds the subfolder to the folder's contents.
                                        if path.is_dir() {
                                            if let Ok(name) = get_name(&path) {
                                                contents.push(Self::UnloadedFolder { path, name });
                                            }
                                        } else {
                                            // Adds the file to the folder's contents.
                                            if let Some(ext) = path.extension() {
                                                if ext == "off" || ext == "ggb" {
                                                    let name = String::from(
                                                        path.file_stem()
                                                            .map(|s| s.to_str())
                                                            .flatten()
                                                            .unwrap_or("none"),
                                                    );

                                                    contents.push(Self::File { path, name });
                                                }
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        println!("Folder read at {:?} failed! Error: {}", path, err)
                                    }
                                }
                            }

                            // Contents of drop down.
                            for lib in contents.iter_mut() {
                                if let Some(file) = lib.show(ui) {
                                    res = Some(file);
                                }
                            }

                            // Opens the folder.
                            *self = Self::LoadedFolder {
                                path,
                                name,
                                contents,
                            };
                        }
                        Err(err) => {
                            println!("Folder read at {:?} failed! Error: {}", path, err);
                        }
                    }
                });

                res
            }
            // Shows a drop-down with all of the files and folders.
            Self::LoadedFolder {
                path: _,
                name,
                contents,
            } => {
                let mut res = None;
                ui.collapsing(name.clone(), |ui| {
                    for lib in contents.iter_mut() {
                        if let Some(file) = lib.show(ui) {
                            res = Some(file);
                        }
                    }
                });

                res
            }
            // Shows a button that loads the file if clicked.
            Self::File { path, name } => {
                if ui.button(name.clone()).clicked() {
                    Some(path.clone())
                } else {
                    None
                }
            }
        }
    }
}