use anyhow::anyhow;
use nutype::nutype;
use regex::Regex;

lazy_static::lazy_static! {
    static ref ALPHANUMERIC_RE: Regex = Regex::new(r"^[a-zA-Z0-9_]+$").unwrap();
}

lazy_static::lazy_static! {
    static ref EMAIL_RE: Regex = Regex::new(r#"^[\w\-\.]+@([\w\-]+\.)+[\w\-]{2,4}$"#).unwrap();
}

fn is_alphanumeric(s: &str) -> anyhow::Result<()> {
    if !ALPHANUMERIC_RE.is_match(s) {
        return Err(anyhow!(
            "Only alphanumeric characters and underscores are allowed."
        ));
    }

    Ok(())
}

fn is_not_too_long(s: &str, max_length: usize) -> anyhow::Result<()> {
    if s.chars().count() > max_length {
        return Err(anyhow!(
            "Too long, maximum length is {max_length} characters."
        ));
    }

    Ok(())
}

fn is_not_empty(s: &str) -> anyhow::Result<()> {
    if s.is_empty() {
        return Err(anyhow!("Cannot be empty."));
    }

    Ok(())
}

fn is_email(s: &str) -> anyhow::Result<()> {
    if s.chars().count() <= 254 && EMAIL_RE.is_match(s) {
        Ok(())
    } else {
        Err(anyhow!("Invalid email."))
    }
}

fn validate_username(s: &str) -> anyhow::Result<()> {
    is_not_empty(s)?;
    is_not_too_long(s, 20)?;
    is_alphanumeric(s)?;

    Ok(())
}

#[nutype(
    sanitize(trim),
    validate(with = validate_username, error = anyhow::Error),
    derive(Deserialize, Serialize, Debug, PartialEq, Clone, Deref)
)]
pub struct Username(String);

#[nutype(
    sanitize(trim, lowercase),
    validate(with = is_email, error = anyhow::Error),
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
    validate(len_char_max = 32, regex =ALPHANUMERIC_RE),
    derive(Deserialize, Serialize, Debug, PartialEq, Clone, Deref)
)]
pub struct AssetName(String);
