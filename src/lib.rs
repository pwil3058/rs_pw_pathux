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

pub extern crate dirs;

use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fs::{DirEntry, FileType, Metadata};
use std::io;
use std::io::Write;
use std::path::{Component, Path, PathBuf, MAIN_SEPARATOR};

#[macro_use]
pub mod str_path;

pub fn split_path_text(text: &str) -> (&str, &str) {
    if let Some(index) = text.rfind(MAIN_SEPARATOR) {
        (&text[..index + 1], &text[index + 1..])
    } else {
        ("", text)
    }
}

pub fn dir_path_text(text: &str) -> &str {
    split_path_text(text).0
}

pub fn file_name_text(text: &str) -> &str {
    split_path_text(text).1
}

pub fn path_to_string(path: &Path) -> String {
    if let Some(path_str) = path.to_str() {
        path_str.to_string()
    } else {
        panic!(
            "File: {} Line: {} : non UniCode file path???",
            file!(),
            line!()
        )
    }
}

pub fn first_subpath_as_string(path: &Path) -> Option<String> {
    for c in path.components() {
        match c {
            Component::RootDir => continue,
            Component::Normal(component) => {
                match component.to_os_string().into_string() {
                    Ok(oss) => return Some(oss),
                    Err(err) => panic!("{:?}: line {:?}: {:?}", file!(), line!(), err),
                };
            }
            Component::Prefix(_) => panic!("Not implemented for Windows"),
            Component::ParentDir => panic!("Illegal component"),
            _ => (),
        }
    }
    None
}

pub fn first_subpath_as_os_string(path: &Path) -> Option<OsString> {
    for c in path.components() {
        match c {
            Component::RootDir => continue,
            Component::Normal(component) => {
                return Some(component.to_os_string());
            }
            Component::Prefix(_) => panic!("Not implemented for Windows"),
            Component::ParentDir => panic!("Illegal component"),
            _ => (),
        }
    }
    None
}

pub fn expand_home_dir(path: &Path) -> Option<PathBuf> {
    if path.is_absolute() {
        return Some(path.to_path_buf());
    } else if !path.exists() {
        let mut components = path.components();
        if let Some(first_component) = components.next() {
            if let Component::Normal(text) = first_component {
                if text == "~" {
                    if let Some(home_dir_path) = dirs::home_dir() {
                        return Some(home_dir_path.join(components.as_path()));
                    }
                }
            }
        }
    };
    None
}

pub fn expand_home_dir_or_mine(path: &Path) -> PathBuf {
    expand_home_dir(path).unwrap_or(path.to_path_buf())
}

pub fn absolute_path_buf(path: &Path) -> PathBuf {
    if path.is_relative() {
        if let Ok(current_dir_path) = env::current_dir() {
            let mut components = path.components();
            if let Some(first_component) = components.next() {
                if let Component::CurDir = first_component {
                    return current_dir_path.join(components.as_path());
                } else {
                    return current_dir_path.join(path);
                }
            } else {
                return current_dir_path;
            }
        } else {
            panic!(
                "File: {} Line: {} : can't find current directory???",
                file!(),
                line!()
            )
        }
    };
    path.to_path_buf()
}

pub fn relative_path_buf(path: &Path) -> Option<PathBuf> {
    if path.is_absolute() {
        if let Ok(current_dir_path) = env::current_dir() {
            if let Ok(rel_path) = path.strip_prefix(&current_dir_path) {
                return Some(rel_path.to_path_buf());
            } else {
                return None;
            }
        } else {
            panic!(
                "File: {} Line: {} : can't find current directory???",
                file!(),
                line!()
            )
        }
    };
    Some(path.to_path_buf())
}

pub fn relative_path_buf_or_mine(path: &Path) -> PathBuf {
    relative_path_buf(path).unwrap_or(path.to_path_buf())
}

#[derive(Debug)]
pub struct UsableDirEntry {
    dir_entry: DirEntry,
    file_type: FileType,
}

impl UsableDirEntry {
    pub fn get_entries<P: AsRef<Path>>(dir_path: &P) -> io::Result<Vec<UsableDirEntry>> {
        usable_dir_entries(dir_path)
    }

    pub fn path(&self) -> PathBuf {
        self.dir_entry.path()
    }

    pub fn file_name(&self) -> String {
        self.dir_entry.file_name().to_string_lossy().into_owned()
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

pub fn usable_dir_entries<P: AsRef<Path>>(dir_path: &P) -> io::Result<Vec<UsableDirEntry>> {
    let dir_path: &Path = dir_path.as_ref();
    let read_dir = dir_path.read_dir()?;
    let mut entries: Vec<UsableDirEntry> = Vec::new();
    for e_entry in read_dir {
        match e_entry {
            Ok(dir_entry) => {
                match dir_entry.metadata() {
                    Ok(metadata) => {
                        let file_type = metadata.file_type();
                        let usable_entry = UsableDirEntry {
                            dir_entry,
                            file_type,
                        };
                        entries.push(usable_entry);
                    }
                    Err(err) => match err.kind() {
                        io::ErrorKind::NotFound => {
                            // we assume that "not found" is due to a race condition and ignore it
                        }
                        io::ErrorKind::PermissionDenied => {
                            // benign so just report it
                            if let Err(wtf) = io::stderr().write_fmt(format_args!(
                                "{:?}: permission denied accessing dir entry",
                                dir_entry
                            )) {
                                // we've got no where to go when writing to stderr fails
                                panic!(
                                    "File: {} Line: {}: {:?}: writing to stderr failed!!!!",
                                    file!(),
                                    line!(),
                                    wtf
                                )
                            }
                        }
                        _ => {
                            panic!("{:?}: {:?}: {:?}", err.kind(), err.description(), dir_entry);
                        }
                    },
                }
            }
            Err(err) => match err.kind() {
                io::ErrorKind::NotFound => {
                    // we assume that "not found" is due to a race condition and ignore it
                }
                io::ErrorKind::PermissionDenied => {
                    // benign so just report it
                    if let Err(wtf) = io::stderr().write_fmt(format_args!(
                        "{:?}: permission denied accessing dir entry",
                        dir_path
                    )) {
                        // we've got no where to go when writing to stderr fails
                        panic!(
                            "File: {} Line: {}: {:?}: writing to stderr failed!!!!",
                            file!(),
                            line!(),
                            wtf
                        )
                    }
                }
                _ => {
                    panic!("{:?}: {:?}: {:?}", err.kind(), err.description(), dir_path);
                }
            },
        }
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_path_text_works() {
        match MAIN_SEPARATOR {
            '/' => {
                assert_eq!(split_path_text("something"), ("", "something"));
                assert_eq!(split_path_text(""), ("", ""));
                assert_eq!(split_path_text("/"), ("/", ""));
                assert_eq!(split_path_text("/something"), ("/", "something"));
                assert_eq!(
                    split_path_text("/something/somethingelse"),
                    ("/something/", "somethingelse")
                );
                assert_eq!(
                    split_path_text("something/somethingelse"),
                    ("something/", "somethingelse")
                );
                assert_eq!(split_path_text("~"), ("", "~"));
            }
            _ => panic!("File: {} Line: {} : new test required"),
        }
    }

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
        assert_eq!(dir_path_text("~/something/somethingelse"), "~/something/");
        assert_eq!(dir_path_text("./something/somethingelse"), "./something/");
    }
}
