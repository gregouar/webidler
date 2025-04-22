pub enum ImageAssetType {
    MonsterPortrait,
    PlayerPortrait,
    ItemIcon,
    SkillIcon,
    WorldBackground,
}

pub fn img_asset(uri: &str) -> String {
    format!("./assets/images/{}", uri)
}
