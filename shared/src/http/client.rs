use nutype::nutype;
use serde::{Deserialize, Serialize};

// #[macro_export]
// macro_rules! define_nutype {
//     (
//         $(#[$meta:meta])*
//         $vis:vis struct $name:ident($inner:ty);

//         sanitize = [$($sanitize:ident),*],
//         validate = [$($validate:expr),*],
//         derive = [$($derive:meta),*],
//         error = $error:ty
//     ) => {
//         #[nutype(
//             sanitize($( $sanitize ),*),
//             validate($( $validate ),*),
//             derive($( $derive ),*)
//         )]
//         $vis struct $name($inner);

//         impl std::convert::TryFrom<String> for $name {
//             type Error = $error;

//             fn try_from(value: String) -> Result<Self, Self::Error> {
//                 Self::try_new(value)
//             }
//         }
//     };
// }

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 20, regex = "^[a-zA-Z0-9_]*$"),
    derive(Deserialize, Serialize, Debug, PartialEq, Clone, Deref)
)]
pub struct Name(String);

// impl TryFrom<String> for Name {
//     type Error = anyhow::Error;

//     fn try_from(value: String) -> Result<Self, Self::Error> {
//         Name::try_new(value).map_err(|e| anyhow::anyhow!(e))
//     }
// }

// impl TryFrom<String> for Name {
//     type Error = NameError;

//     fn try_from(value: String) -> Result<Self, Self::Error> {
//         Self::try_new(value)
//     }
// }

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
