use std::sync::LazyLock;

use regex::Regex;

pub const ERROR_ALREADY_EXISTS: &str = "already-exists";

pub static REGEX_USERNAME: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\A[-_.]?([[:alnum:]]+[-_.]?)+\z").unwrap());
