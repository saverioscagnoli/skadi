pub struct Templates;

impl Templates {
    pub const DEFAULT_CONFIG: &'static str = include_str!("../../templates/config.default.json");
    pub const LICENSE: &'static str = include_str!("../../templates/LICENSE");
}
