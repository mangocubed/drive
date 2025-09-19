use std::sync::LazyLock;

pub const COPYRIGHT: &str = "© 2025 Mango³ Group";

pub const PRIVACY_URL: &str = "https://mango3.app/#privacy";
pub const TERMS_URL: &str = "https://mango3.app/#terms";

pub static SOURCE_CODE_URL: LazyLock<String> =
    LazyLock::new(|| format!("{}/tree/{}", env!("CARGO_PKG_REPOSITORY"), env!("GIT_REV_SHORT")));
