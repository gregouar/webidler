pub fn img_asset(uri: &str) -> String {
    format!("./assets/images/{uri}")
}

pub fn screenshot_asset(uri: &str) -> String {
    format!("./assets/screenshots/{uri}")
}
