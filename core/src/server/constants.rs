use std::borrow::Cow;
use std::sync::LazyLock;

use file_format::FileFormat;
use validator::ValidationError;

use regex::Regex;

pub const ALLOWED_FILE_FORMATS: [FileFormat; 3] = [
    FileFormat::GraphicsInterchangeFormat,
    FileFormat::JointPhotographicExpertsGroup,
    FileFormat::PortableNetworkGraphics,
];

pub static ERROR_IS_TOO_LARGE: LazyLock<ValidationError> =
    LazyLock::new(|| ValidationError::new("too-large").with_message(Cow::Borrowed("Is too large")));

pub const METADATA_TOTAL_SPACE: &str = "space_quota";

pub static REGEX_FILE_NAME: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"\A[^\/:*?"<>|]+\z"#).unwrap());
