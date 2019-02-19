// Copyright 2019 Peter Williams <pwil3058@gmail.com>
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

//! A module to provide a mechanism for doing file path operations
//! on Strings and str.

pub use std::convert::From;
pub use std::env;
pub use std::ffi::OsStr;
pub use std::io;
pub use std::path::{Component, Path, PathBuf, Prefix, MAIN_SEPARATOR};

pub use dirs;

#[macro_export]
macro_rules! str_path_file_name {
    ( $s:expr ) => {
        match Path::new($s).file_name() {
            Some(os_str) => Some(os_str.to_string_lossy().into_owned()),
            None => None,
        }
    };
}

#[macro_export]
macro_rules! str_path_parent {
    ( $s:expr ) => {
        match Path::new($s).parent() {
            Some(path) => Some(path.to_string_lossy().into_owned()),
            None => None,
        }
    };
}

#[macro_export]
macro_rules! str_path_components {
    ( $s:expr ) => {{
        Path::new($s).components().enumerate().map(|(i, c)| {
            if i == 0 && c == Component::Normal(OsStr::new("~")) {
                StrPathComponent::HomeDir
            } else {
                StrPathComponent::from(c)
            }
        })
    }};
}

#[macro_export]
macro_rules! str_path_is_absolute {
    ( $s:expr ) => {{
        match str_path_components!($s).take(1).next() {
            Some(StrPathComponent::HomeDir) => false,
            _ => Path::new($s).is_absolute(),
        }
    }};
}

#[macro_export]
macro_rules! str_path_is_relative {
    ( $s:expr ) => {{
        match str_path_components!($s).take(1).next() {
            Some(StrPathComponent::HomeDir) => false,
            _ => Path::new($s).is_relative(),
        }
    }};
}

#[macro_export]
macro_rules! str_path_is_relative_to_home {
    ( $s:expr ) => {{
        match str_path_components!($s).take(1).next() {
            Some(StrPathComponent::HomeDir) => true,
            _ => false,
        }
    }};
}

#[macro_export]
macro_rules! str_path_absolute {
    ( $s:expr ) => {{
        if str_path_is_absolute!($s) {
            Ok($s.to_string())
        } else if str_path_is_relative!($s) {
            match env::current_dir() {
                Ok(mut cur_dir) => {
                    for c in Path::new($s)
                        .components()
                        .skip_while(|c| *c == Component::CurDir)
                    {
                        cur_dir.push(c)
                    }
                    Ok(cur_dir.to_string_lossy().into_owned())
                }
                Err(err) => Err(err),
            }
        } else {
            match dirs::home_dir() {
                Some(mut home_dir) => {
                    for c in Path::new($s).components().skip(1) {
                        home_dir.push(c)
                    }
                    Ok(home_dir.to_string_lossy().into_owned())
                }
                None => Err(io::Error::new(
                    io::ErrorKind::Other,
                    "could not find home directory",
                )),
            }
        }
    }};
}

#[macro_export]
macro_rules! str_path_simple_relative {
    ( $s:expr ) => {{
        match env::current_dir() {
            Ok(curr_dir) => match str_path_absolute!($s) {
                Ok(abs_path) => match Path::new(&abs_path).strip_prefix(curr_dir) {
                    Ok(path) => Ok(path.to_string_lossy().into_owned()),
                    Err(err) => Err(io::Error::new(io::ErrorKind::Other, err)),
                },
                Err(err) => Err(err),
            },
            Err(err) => Err(err),
        }
    }};
}

pub trait StringPathBuf {
    fn path_push(&mut self, path: &str);
}

impl StringPathBuf for String {
    fn path_push(&mut self, path: &str) {
        if cfg!(target_os = "windows") {
            let mut new_path = PathBuf::new();
            new_path.push(self.clone());
            new_path.push(path);
            self.clear();
            self.push_str(&new_path.to_string_lossy().into_owned());
        } else if str_path_is_absolute!(path) {
            self.clear();
            self.push_str(path);
        } else {
            self.push(MAIN_SEPARATOR);
            self.push_str(path);
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum StrPathPrefix {
    Verbatim(String),
    VerbatimUNC(String, String),
    VerbatimDisk(u8),
    DeviceNS(String),
    UNC(String, String),
    Disk(u8),
}

impl<'a> From<Prefix<'a>> for StrPathPrefix {
    fn from(prefix: Prefix) -> Self {
        match prefix {
            Prefix::Verbatim(os_str) => {
                StrPathPrefix::Verbatim(os_str.to_string_lossy().into_owned())
            }
            Prefix::VerbatimUNC(os_str1, os_str2) => StrPathPrefix::VerbatimUNC(
                os_str1.to_string_lossy().into_owned(),
                os_str2.to_string_lossy().into_owned(),
            ),
            Prefix::VerbatimDisk(u8) => StrPathPrefix::VerbatimDisk(u8),
            Prefix::DeviceNS(os_str) => {
                StrPathPrefix::DeviceNS(os_str.to_string_lossy().into_owned())
            }
            Prefix::UNC(os_str1, os_str2) => StrPathPrefix::UNC(
                os_str1.to_string_lossy().into_owned(),
                os_str2.to_string_lossy().into_owned(),
            ),
            Prefix::Disk(u8) => StrPathPrefix::Disk(u8),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum StrPathComponent {
    Prefix(StrPathPrefix),
    RootDir,
    HomeDir,
    CurDir,
    ParentDir,
    Normal(String),
}

impl<'a> From<Component<'a>> for StrPathComponent {
    fn from(component: Component) -> Self {
        match component {
            Component::Prefix(prefix) => StrPathComponent::Prefix(prefix.kind().into()),
            Component::RootDir => StrPathComponent::RootDir,
            Component::CurDir => StrPathComponent::CurDir,
            Component::ParentDir => StrPathComponent::ParentDir,
            Component::Normal(os_str) => {
                StrPathComponent::Normal(os_str.to_string_lossy().into_owned())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_family = "unix")]
    #[test]
    fn str_path_works() {
        assert_eq!(
            str_path_file_name!("/home/peter"),
            Some("peter".to_string())
        );
        assert_eq!(
            str_path_file_name!(&"/home/peter".to_string()),
            Some("peter".to_string())
        );
        assert_eq!(str_path_file_name!("/home"), Some("home".to_string()));
        assert_eq!(str_path_file_name!("/"), None);
        assert_eq!(str_path_file_name!("peter"), Some("peter".to_string()));
        assert_eq!(str_path_file_name!("home/"), Some("home".to_string()));

        assert_eq!(str_path_parent!("/home/peter"), Some("/home".to_string()));
        assert_eq!(
            str_path_parent!(&"/home/peter".to_string()),
            Some("/home".to_string())
        );
        assert_eq!(
            str_path_parent!(&"/home".to_string()),
            Some("/".to_string())
        );
        assert_eq!(str_path_parent!(&"/".to_string()), None);
        assert_eq!(str_path_parent!(&"peter".to_string()), Some("".to_string()));

        let mut components = str_path_components!("/home/peter/SRC");
        assert_eq!(components.next(), Some(StrPathComponent::RootDir));
        assert_eq!(
            components.next(),
            Some(StrPathComponent::Normal("home".to_string()))
        );
        assert_eq!(
            components.next(),
            Some(StrPathComponent::Normal("peter".to_string()))
        );
        assert_eq!(
            components.next(),
            Some(StrPathComponent::Normal("SRC".to_string()))
        );
        assert_eq!(components.next(), None);
        let mut components = str_path_components!("./peter/SRC");
        assert_eq!(components.next(), Some(StrPathComponent::CurDir));
        let mut components = str_path_components!("~/SRC");
        assert_eq!(components.next(), Some(StrPathComponent::HomeDir));
        assert_eq!(
            components.next(),
            Some(StrPathComponent::Normal("SRC".to_string()))
        );
        assert_eq!(components.next(), None);

        assert!(str_path_is_absolute!("/home"));
        assert!(!str_path_is_absolute!("~/SRC"));

        assert!(!str_path_is_relative!("/home"));
        assert!(!str_path_is_relative!("~/SRC"));
        assert!(str_path_is_relative!("./SRC"));
        assert!(str_path_is_relative!("SRC"));
        assert!(str_path_is_relative_to_home!("~/SRC"));

        assert_eq!(
            str_path_absolute!("./SRC").unwrap(),
            "/home/peter/SRC/GITHUB/rs_gwsm_git.git/pw_pathux/SRC".to_string()
        );
        assert_eq!(
            str_path_absolute!("/home/peter/SRC").unwrap(),
            "/home/peter/SRC".to_string()
        );
        assert_eq!(
            str_path_absolute!("~/SRC").unwrap(),
            "/home/peter/SRC".to_string()
        );

        assert_eq!(
            str_path_simple_relative!("./SRC").unwrap(),
            "SRC".to_string()
        );
        assert_eq!(
            str_path_simple_relative!("/home/peter/SRC/GITHUB/rs_gwsm_git.git/pw_pathux/SRC")
                .unwrap(),
            "SRC".to_string()
        );
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn string_path_works() {
        let mut path = "/home".to_string();
        path.path_push("peter");
        assert_eq!(path, "/home/peter".to_string());

        path = "peter".to_string();
        path.path_push("/home/peter/SRC");
        assert_eq!(path, "/home/peter/SRC".to_string());
    }
}
