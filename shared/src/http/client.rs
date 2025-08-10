use nutype::nutype;
use serde::{Deserialize, Serialize};

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 20, regex = "^[a-zA-Z0-9_]*$"),
    derive(Deserialize, Serialize, Debug, PartialEq, Clone, Deref)
)]
pub struct Name(String);

#[nutype(
    sanitize(trim, lowercase),
    validate(len_char_max = 254, regex = r#"^[\w\-\.]+@([\w\-]+\.)+[\w\-]{2,4}$"#),
    derive(Deserialize, Serialize, Debug, PartialEq, Clone, Deref)
)]
pub struct Email(String);

#[nutype(
    sanitize(trim),
    validate(len_char_min = 5, len_char_max = 128),
    derive(Deserialize, Serialize, Debug, PartialEq, Clone, Deref)
)]
pub struct Password(String);

#[nutype(
    sanitize(trim),
    validate(len_char_max = 32, regex = r"^[a-zA-Z0-9_]*$"),
    derive(Deserialize, Serialize, Debug, PartialEq, Clone, Deref)
)]
pub struct AssetName(String);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SignUpRequest {
    pub captcha_token: String,
    pub username: Name,
    pub email: Option<Email>,
    pub password: Password,
    pub accepted_terms: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SignInRequest {
    pub captcha_token: String,
    pub username: Name,
    pub password: Password,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateCharacterRequest {
    pub name: Name,
    pub portrait: AssetName,
}
