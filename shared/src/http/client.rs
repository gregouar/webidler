use nutype::nutype;
use regex::Regex;
use serde::{Deserialize, Serialize};

// TODO: move

fn is_alphanumeric(s: &str) -> Result<(), String> {
    lazy_static::lazy_static! {
        static ref RE: Regex = Regex::new(r"^[a-zA-Z0-9_]+$").unwrap();
    }

    if RE.is_match(s) {
        Ok(())
    } else {
        Err("Only alphanumeric characters and underscores are allowed.".to_string())
    }
}

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 20, regex=r"^[a-zA-Z0-9_]+$"),
    // validate(with = is_alphanumeric, error=String),
    derive(Deserialize, Serialize, Debug, PartialEq, Clone, Deref)
)]
pub struct Name(String);

fn is_email(s: &str) -> Result<(), String> {
    lazy_static::lazy_static! {
        static ref RE: Regex = Regex::new(r#"^[\w\-\.]+@([\w\-]+\.)+[\w\-]{2,4}$"#).unwrap();
    }

    if RE.is_match(s) {
        Ok(())
    } else {
        Err("Invalid email.".to_string())
    }
}

#[nutype(
    sanitize(trim, lowercase),
    validate(len_char_max = 254, regex=r#"^[\w\-\.]+@([\w\-]+\.)+[\w\-]{2,4}$"#),
    // validate(with = is_email, error=String),
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
