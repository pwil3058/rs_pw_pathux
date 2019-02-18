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
pub use std::ffi::OsStr;
pub use std::path::{Component, Path, Prefix};

#[macro_export]
macro_rules! path_file_name {
    ( $s:expr ) => {
        match Path::new($s).file_name() {
            Some(os_str) => Some(os_str.to_string_lossy().into_owned()),
            None => None,
        }
    };
}

#[macro_export]
macro_rules! path_parent {
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
        !str_path_is_absolute!($s)
    }};
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

    #[test]
    fn str_path_works() {
        assert_eq!(path_file_name!("/home/peter"), Some("peter".to_string()));
        assert_eq!(
            path_file_name!(&"/home/peter".to_string()),
            Some("peter".to_string())
        );
        assert_eq!(path_file_name!("/home"), Some("home".to_string()));
        assert_eq!(path_file_name!("/"), None);
        assert_eq!(path_file_name!("peter"), Some("peter".to_string()));
        assert_eq!(path_file_name!("home/"), Some("home".to_string()));

        assert_eq!(path_parent!("/home/peter"), Some("/home".to_string()));
        assert_eq!(
            path_parent!(&"/home/peter".to_string()),
            Some("/home".to_string())
        );
        assert_eq!(path_parent!(&"/home".to_string()), Some("/".to_string()));
        assert_eq!(path_parent!(&"/".to_string()), None);
        assert_eq!(path_parent!(&"peter".to_string()), Some("".to_string()));

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
        assert!(str_path_is_relative!("~/SRC"));
    }
}
