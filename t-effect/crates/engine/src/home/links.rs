//! Community and project links (shown in Help ▸, the About dialog, the start screen and the
//! header's Discord button).

pub const APP_NAME: &str = "EffectCraft";
pub const APP_SLUG: &str = "effectcraft";
// TitanOS-Office: no ArtCraft links (fork rule). "Discord" and the app page lead to TitanOS-Office,
// the website to TitanOS, GitHub (and the docs) to the original EffectCraft repository.
pub const DISCORD: &str = "https://github.com/ra5on/TitanOS-Office";
pub const WEBSITE: &str = "https://github.com/ra5on/TitanOS";
pub const APP_PAGE: &str = "https://github.com/ra5on/TitanOS-Office";
pub const GITHUB: &str = "https://github.com/storytold/effectcraft";
pub const ISSUES: &str = "https://github.com/ra5on/TitanOS-Office/issues";

/// Sibling apps of the ArtCraft family: (name, what it is, GitHub repo, app page).
pub const SIBLINGS: &[(&str, &str)] = &[
    ("PhotoCraft", "photocraft"),
    ("VectorCraft", "vectorcraft"),
    ("FilmCraft", "filmcraft"),
    ("LightCraft", "lightcraft"),
    ("PdfCraft", "pdfcraft"),
    ("DesignCraft", "designcraft"),
];

pub fn github_of(slug: &str) -> String {
    format!("https://github.com/storytold/{slug}")
}
pub fn page_of(slug: &str) -> String {
    format!("https://github.com/storytold/{slug}")
}
