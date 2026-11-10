//! Help → community and project links (the ArtCraft Discord, website, this app's page and its
//! GitHub repository). The commands return the URL; the frontend opens it.

use serde_json::{Value, json};

use super::{CommandSpec, Result, always, cmd};

/// The app's id on getartcraft.com and GitHub.
pub const APP_ID: &str = "vectorcraft";
// TitanOS-Office: no ArtCraft links (fork rule). "discord" is TitanOS-Office, "website" is
// TitanOS, "appPage" is the original VectorCraft repository.
pub const DISCORD_URL: &str = "https://github.com/ra5on/TitanOS-Office";
pub const WEBSITE_URL: &str = "https://github.com/ra5on/TitanOS";

/// The original project's repository.
pub fn app_page_url() -> String {
    format!("https://github.com/storytold/{APP_ID}")
}

/// This app's source repository.
pub fn github_url() -> String {
    "https://github.com/ra5on/TitanOS-Office".to_string()
}

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(query "help.discord", "TitanOS-Office on GitHub", ["Help"], None, "{} → {url} TitanOS-Office", always, |_, _| url(DISCORD_URL.into())),
        cmd!(query "help.website", "TitanOS on GitHub", ["Help"], None, "{} → {url}", always, |_, _| url(WEBSITE_URL.into())),
        cmd!(query "help.appPage", "Original: VectorCraft on GitHub", ["Help"], None, "{} → {url} the original project", always, |_, _| url(app_page_url())),
        cmd!(query "help.github", "T-Vector on GitHub", ["Help"], None, "{} → {url} source code, issues and releases", always, |_, _| url(github_url())),
        cmd!(query "help.links", "Links", [], None, "{} → {discord, website, appPage, github}", always, |_, _| {
            Ok(json!({ "discord": DISCORD_URL, "website": WEBSITE_URL, "appPage": app_page_url(), "github": github_url() }))
        }),
    ]
}

fn url(u: String) -> Result<Value> {
    Ok(json!({ "url": u }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Session;

    #[test]
    fn links() {
        let mut s = Session::new();
        assert_eq!(s.execute("help.discord", &json!({})).unwrap()["url"], "https://discord.gg/artcraft");
        assert_eq!(s.execute("help.appPage", &json!({})).unwrap()["url"], "https://getartcraft.com/apps/vectorcraft");
        assert_eq!(s.execute("help.github", &json!({})).unwrap()["url"], "https://github.com/storytold/vectorcraft");
        assert_eq!(s.execute("help.links", &json!({})).unwrap()["website"], "https://getartcraft.com");
    }
}
