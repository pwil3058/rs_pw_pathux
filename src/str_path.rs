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

use std::string::ToString;

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

#[macro_export]
macro_rules! str_path_simple_relative_home {
    ( $s:expr ) => {{
        match dirs::home_dir() {
            Some(home_dir) => match str_path_absolute!($s) {
                Ok(abs_path) => match Path::new(&abs_path).strip_prefix(home_dir) {
                    Ok(path) => {
                        let mut home = PathBuf::new();
                        home.push("~");
                        home.push(&path);
                        Ok(home.to_string_lossy().into_owned())
                    },
                    Err(err) => Err(io::Error::new(io::ErrorKind::Other, err)),
                },
                Err(err) => Err(err),
            },
            None =>Err(io::Error::new(
                    io::ErrorKind::Other,
                    "could not find home directory",
                )),
        }
    }};
}

#[macro_export]
macro_rules! str_path_join {
    ( $s1:expr, $s2:expr ) => {{
        Path::new($s1)
            .join(Path::new($s2))
            .to_string_lossy()
            .into_owned()
    }};
}

pub fn str_path_current_dir() -> io::Result<String> {
    match env::current_dir() {
        Ok(path_buf) => Ok(path_buf.to_string_lossy().into_owned()),
        Err(e) => Err(e),
    }
}

pub fn str_path_current_dir_or_panic() -> String {
    str_path_current_dir().expect("Could not find current directory.")
}

pub fn str_path_current_dir_rel_home() -> io::Result<String> {
    match env::current_dir() {
        Ok(path_buf) => str_path_simple_relative_home!(&path_buf.to_string_lossy().into_owned()),
        Err(e) => Err(e),
    }
}

pub fn str_path_current_dir_or_rel_home_panic() -> String {
    str_path_current_dir_rel_home().expect("Could not find current directory.")
}

pub trait StrPath {
    fn path_absolute(&self) -> io::Result<String>;
    fn path_components(&self) -> Vec<StrPathComponent>;
    fn path_is_absolute(&self) -> bool;
    fn path_is_dir(&self) -> bool;
    fn path_is_file(&self) -> bool;
    fn path_is_relative(&self) -> bool;
    fn path_is_relative_to_home(&self) -> bool;
    fn path_file_name(&self) -> Option<String>;
    fn path_join(&self, other: &str) -> String;
    fn path_parent(&self) -> Option<String>;
    fn path_simple_relative(&self) -> io::Result<String>;
    fn path_starts_with(&self, prefix: &str) -> bool;
}

impl StrPath for str {
    fn path_absolute(&self) -> io::Result<String> {
        str_path_absolute!(self)
    }

    fn path_components(&self) -> Vec<StrPathComponent> {
        str_path_components!(self).collect()
    }

    fn path_is_absolute(&self) -> bool {
        str_path_is_absolute!(self)
    }

    fn path_is_dir(&self) -> bool {
        Path::new(self).is_dir()
    }

    fn path_is_file(&self) -> bool {
        Path::new(self).is_file()
    }

    fn path_is_relative(&self) -> bool {
        str_path_is_relative!(self)
    }

    fn path_is_relative_to_home(&self) -> bool {
        str_path_is_relative_to_home!(self)
    }

    fn path_file_name(&self) -> Option<String> {
        str_path_file_name!(self)
    }

    fn path_join(&self, other: &str) -> String {
        str_path_join!(self, other)
    }

    fn path_parent(&self) -> Option<String> {
        str_path_parent!(self)
    }

    fn path_simple_relative(&self) -> io::Result<String> {
        str_path_simple_relative!(self)
    }

    fn path_starts_with(&self, prefix: &str) -> bool {
        Path::new(self).starts_with(Path::new(prefix))
    }
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

impl ToString for StrPathPrefix {
    fn to_string(&self) -> String {
        match self {
            StrPathPrefix::Verbatim(string) => format!(r"\\?\{}", string),
            StrPathPrefix::VerbatimUNC(server, share) => format!(r"\\?\UNC\{}\{}", server, share),
            StrPathPrefix::VerbatimDisk(vid) => format!(r"\\?\{}:\", vid.to_string()),
            StrPathPrefix::DeviceNS(device) => format!(r"\\.\{}", device),
            StrPathPrefix::UNC(server, share) => format!(r"\\{}\{}", server, share),
            StrPathPrefix::Disk(id) => format!(r"{}:", id.to_string()),
        }
    }
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

impl ToString for StrPathComponent {
    fn to_string(&self) -> String {
        match self {
            StrPathComponent::Prefix(stp) => stp.to_string(),
            StrPathComponent::RootDir => MAIN_SEPARATOR.to_string(),
            StrPathComponent::HomeDir => "~".to_string(),
            StrPathComponent::CurDir => ".".to_string(),
            StrPathComponent::ParentDir => "..".to_string(),
            StrPathComponent::Normal(string) => string.clone(),
        }
    }
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

impl StrPathComponent {
    pub fn is_cur_dir(&self) -> bool {
        match self {
            StrPathComponent::CurDir => true,
            _ => false,
        }
    }

    pub fn is_home_dir(&self) -> bool {
        match self {
            StrPathComponent::HomeDir => true,
            _ => false,
        }
    }

    pub fn is_normal(&self) -> bool {
        match self {
            StrPathComponent::Normal(_) => true,
            _ => false,
        }
    }
}

pub trait ToStringPath {
    fn to_string_path(&self) -> String;
}

impl ToStringPath for Path {
    fn to_string_path(&self) -> String {
        self.to_string_lossy().to_owned().to_string()
    }
}

impl ToStringPath for PathBuf {
    fn to_string_path(&self) -> String {
        self.to_string_lossy().to_owned().to_string()
    }
}

impl ToStringPath for [StrPathComponent] {
    fn to_string_path(&self) -> String {
        let mut path_buf = PathBuf::new();
        for component in self.iter() {
            path_buf.push(component.to_string());
        }
        path_buf.to_string_lossy().to_owned().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_family = "unix")]
    #[test]
    fn str_path_macros_work() {
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

        assert_eq!(
            str_path_join!("/home/peter/SRC", "GITHUB/rs_gwsm_git.git/pw_pathux/SRC"),
            "/home/peter/SRC/GITHUB/rs_gwsm_git.git/pw_pathux/SRC".to_string()
        )
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

    #[test]
    fn str_path_works() {
        assert!("/home".path_is_absolute());
        assert!("/home".to_string().path_is_absolute());
    }

    #[test]
    fn str_path_components_work() {
        assert_eq!(
            Path::new("/home/peter/SRC").to_string_path(),
            "/home/peter/SRC".to_string()
        );
        let mut path_buf = PathBuf::new();
        path_buf.push("/home/peter/SRC");
        assert_eq!(path_buf.to_string_path(), "/home/peter/SRC".to_string());

        let components = "/home/peter/SRC".path_components();
        assert_eq!(components[2..].to_string_path(), "peter/SRC".to_string());
        let components = "home/peter/SRC".path_components();
        assert_eq!(components[1..].to_string_path(), "peter/SRC".to_string());
    }
}
