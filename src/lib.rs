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
use std::io;
use std::path::{PathBuf, MAIN_SEPARATOR};


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
