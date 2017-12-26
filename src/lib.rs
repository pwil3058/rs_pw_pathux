// Copyright 2017 Peter Williams <pwil3058@gmail.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::env;
use std::error::Error;
use std::fs::{DirEntry, FileType, Metadata};
use std::io;
use std::io::{Write};
use std::path::{Path, PathBuf, MAIN_SEPARATOR};


pub fn dir_path_text(text: &str) -> String {
    if let Some(index) = text.rfind(MAIN_SEPARATOR) {
        text[..index + 1].to_string()
    } else {
        "".to_string()
    }
}

pub fn abs_dir_path(text: &str) -> io::Result<PathBuf> {
    if text.len() == 0 {
        env::current_dir()
    } else if text.starts_with('~') {
        if let Some(index) = text.find(MAIN_SEPARATOR) {
            if index == 1 {
                if let Some(mut dir_path) = env::home_dir() {
                    dir_path.push(&text[index + 1..]);
                    Ok(dir_path)
                } else {
                    Err(io::Error::new(io::ErrorKind::Other, "Could not find home directory"))
                }
            } else {
                let msg = format!("Could not find {}'s home directory", &text[1..index]);
                Err(io::Error::new(io::ErrorKind::Other, msg))
            }
        } else {
            if let Some(dir_path) = env::home_dir() {
                Ok(dir_path)
            } else {
                Err(io::Error::new(io::ErrorKind::Other, "Could not find home directory"))
            }
        }
    } else if text.starts_with('.') {
        PathBuf::from(text).canonicalize()
    } else {
        let dir_path = PathBuf::from(text);
        if dir_path.is_absolute() {
            Ok(dir_path)
        } else {
            Err(io::Error::new(io::ErrorKind::Other, dir_path.to_str().unwrap()))
        }
    }
}

#[derive(Debug)]
pub struct UsableDirEntry {
    dir_entry: DirEntry,
    file_type: FileType,
}

impl UsableDirEntry {
    pub fn path(&self) -> PathBuf {
        self.dir_entry.path()
    }

    pub fn file_name(&self) -> String {
        if let Ok(file_name) = self.dir_entry.file_name().into_string() {
            file_name
        } else {
            panic!("File: {} Line: {} : \"{:?}\" badly designed OS", file!(), line!(), self.dir_entry.file_name())
        }
    }

    pub fn is_dir(&self) -> bool {
        self.file_type.is_dir()
    }

    pub fn is_file(&self) -> bool {
        self.file_type.is_file()
    }

    pub fn is_symlink(&self) -> bool {
        self.file_type.is_symlink()
    }

    pub fn file_type(&self) -> FileType {
        self.file_type
    }

    pub fn metadata(&self) -> io::Result<Metadata> {
        self.dir_entry.metadata()
    }
}

pub fn usable_dir_entries(dir_path: &Path) -> io::Result<Vec<UsableDirEntry>> {
    let read_dir = dir_path.read_dir()?;
    let mut entries: Vec<UsableDirEntry> = Vec::new();
    for e_entry in read_dir {
        match e_entry {
            Ok(dir_entry) => {
                match dir_entry.metadata() {
                    Ok(metadata) => {
                        let file_type = metadata.file_type();
                        let usable_entry = UsableDirEntry{dir_entry, file_type};
                        entries.push(usable_entry);
                    },
                    Err(err) => match err.kind() {
                        io::ErrorKind::NotFound => {
                            // we assume that "not found" is due to a race condition and ignore it
                        },
                        io::ErrorKind::PermissionDenied => {
                            // benign so just report it
                            if let Err(wtf) = io::stderr().write_fmt(format_args!("{:?}: permission denied accessing dir entry", dir_entry)) {
                                // we've got no where to go when writing to stderr fails
                                panic!("File: {} Line: {}: {:?}: writing to stderr failed!!!!", file!(), line!(), wtf)
                            }
                        },
                        _ => {
                            panic!("{:?}: {:?}: {:?}", err.kind(), err.description(), dir_entry);
                        }
                    }
                }
            },
            Err(err) => match err.kind() {
                io::ErrorKind::NotFound => {
                    // we assume that "not found" is due to a race condition and ignore it
                },
                io::ErrorKind::PermissionDenied => {
                    // benign so just report it
                    if let Err(wtf) = io::stderr().write_fmt(format_args!("{:?}: permission denied accessing dir entry", dir_path)) {
                        // we've got no where to go when writing to stderr fails
                        panic!("File: {} Line: {}: {:?}: writing to stderr failed!!!!", file!(), line!(), wtf)
                    }
                },
                _ => {
                    panic!("{:?}: {:?}: {:?}", err.kind(), err.description(), dir_path);
                }
            }
        }
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dir_path_text_works() {
        assert_eq!(dir_path_text("something"), "");
        assert_eq!(dir_path_text(""), "");
        assert_eq!(dir_path_text("/"), "/");
        assert_eq!(dir_path_text("/something"), "/");
        assert_eq!(dir_path_text("/something/somethingelse"), "/something/");
        assert_eq!(dir_path_text("something/somethingelse"), "something/");
        assert_eq!(dir_path_text("~/"), "~/");
        assert_eq!(dir_path_text("./"), "./");
        assert_eq!(dir_path_text("~/something"), "~/");
        assert_eq!(dir_path_text("./something"), "./");
    }
}
