use std::collections::HashSet;
use std::fmt::Display;
use std::fs::OpenOptions;
use std::io::{Read, Write};

use fake::Fake;
use fake::faker::internet::en::{FreeEmail, Username};
use rand::rng;

fn unique_fake<T, F>(prefix: &str, fake_fn: F) -> T
where
    F: Fn() -> T,
    T: Display,
{
    let file_path = std::env::temp_dir().join("used_fakes");

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .read(true)
        .open(&file_path)
        .expect("Could not open file");

    let mut file_content = String::new();

    let _ = file.read_to_string(&mut file_content);

    let mut lines = file_content
        .lines()
        .map(|line| line.to_owned())
        .collect::<HashSet<String>>();

    if lines.len() > 100 {
        for line in lines.clone().iter().take(lines.len() - 100) {
            lines.remove(line);
        }
    }

    let _ = file.set_len(0);

    for line in &lines {
        let _ = file.write_all(format!("{line}\n").as_bytes());
    }

    let mut fake = fake_fn();

    while !lines.insert(format!("{prefix}_{fake}")) {
        fake = fake_fn();
    }

    let _ = file.write_all(format!("{prefix}_{fake}\n").as_bytes());

    fake
}

fn fake_email() -> String {
    unique_fake("email", || FreeEmail().fake_with_rng(&mut rng()))
}

pub fn fake_username() -> String {
    unique_fake("username", || {
        let mut username: String = Username().fake_with_rng(&mut rng());
        username.truncate(16);
        username
    })
}
