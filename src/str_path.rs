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

pub use std::path::Path;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn str_path_works() {
        assert_eq!(path_file_name!("/home/peter"), Some("peter".to_string()));
        assert_eq!(path_file_name!(&"/home/peter".to_string()), Some("peter".to_string()));
        assert_eq!(path_file_name!("/home"), Some("home".to_string()));
        assert_eq!(path_file_name!("/"), None);
        assert_eq!(path_file_name!("peter"), Some("peter".to_string()));
        assert_eq!(path_file_name!("home/"), Some("home".to_string()));

        assert_eq!(path_parent!("/home/peter"), Some("/home".to_string()));
        assert_eq!(path_parent!(&"/home/peter".to_string()), Some("/home".to_string()));
        assert_eq!(path_parent!(&"/home".to_string()), Some("/".to_string()));
        assert_eq!(path_parent!(&"/".to_string()), None);
        assert_eq!(path_parent!(&"peter".to_string()), Some("".to_string()));
    }
}
