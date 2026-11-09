//! Project links (Help menu, About, start screen, CLI, MCP). TitanOS-Office: no ArtCraft links
//! (fork rule); "discord" is TitanOS-Office, "website" TitanOS, "appPage" the original DesignCraft.

use serde_json::{Value, json};

/// The app's slug on getartcraft.com and GitHub.
pub const APP: &str = "designcraft";
/// TitanOS-Office (was the community Discord).
pub const DISCORD: &str = "https://github.com/ra5on/TitanOS-Office";
/// TitanOS (was the website).
pub const WEBSITE: &str = "https://github.com/ra5on/TitanOS";
/// The original DesignCraft repository (was the app page).
pub const APP_PAGE: &str = "https://github.com/storytold/designcraft";
/// The source repository.
pub const GITHUB: &str = "https://github.com/ra5on/TitanOS-Office";
/// Where to report bugs and request features.
pub const ISSUES: &str = "https://github.com/ra5on/TitanOS-Office/issues";

/// All links as JSON (the `app.links` command).
pub fn all() -> Value {
    json!({
        "discord": DISCORD,
        "website": WEBSITE,
        "appPage": APP_PAGE,
        "github": GITHUB,
        "issues": ISSUES,
    })
}

/// A link by key (`discord`, `website`, `appPage`, `github`, `issues`).
pub fn get(key: &str) -> Option<&'static str> {
    Some(match key {
        "discord" => DISCORD,
        "website" => WEBSITE,
        "appPage" => APP_PAGE,
        "github" => GITHUB,
        "issues" => ISSUES,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_follow_the_app_slug() {
        assert!(APP_PAGE.ends_with(&format!("/apps/{APP}")));
        assert!(GITHUB.ends_with(&format!("/storytold/{APP}")));
        assert!(ISSUES.starts_with(GITHUB));
        for k in ["discord", "website", "appPage", "github", "issues"] {
            assert!(get(k).is_some_and(|u| u.starts_with("https://")), "{k}");
            assert_eq!(all()[k], get(k).unwrap());
        }
    }
}
