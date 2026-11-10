//! Interface language (TitanOS-Office). The original has English text only, written where it is
//! drawn. This is the smallest mechanism that changes that: the English text is the key, [`tr`]
//! returns the row of the current language or the text itself, so anything without a row stays
//! English. So far the ribbon goes through it (its tabs, and the buttons, menu rows and tooltips drawn by the
//! helpers in `widgets` and `ribbon`); the File view, panes and dialogs still draw their own English.
//!
//! The language is set once at start ([`set_language`]); the web build follows the browser.

use std::collections::HashMap;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

/// Whether the interface is drawn in German (English otherwise).
static GERMAN: AtomicBool = AtomicBool::new(false);

/// Whether `locale` (BCP 47, such as `de`, `de-AT` or `en-US`) is a German one.
fn is_german(locale: &str) -> bool {
    locale.trim().split(['-', '_', '.', '@']).next().is_some_and(|primary| primary.eq_ignore_ascii_case("de"))
}

/// Choose the interface language from a locale: German for every `de` locale, English for
/// anything else.
pub fn set_language(locale: &str) {
    GERMAN.store(is_german(locale), Ordering::Relaxed);
}

/// The code of the language in force: `"de"` or `"en"`.
pub fn language() -> &'static str {
    if GERMAN.load(Ordering::Relaxed) { "de" } else { "en" }
}

/// The German row for `text`, if there is one.
fn german(text: &str) -> Option<&'static str> {
    static INDEX: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    INDEX.get_or_init(|| DE.iter().copied().collect()).get(text).copied()
}

/// `text` in the current language; `text` itself when there is no row for it.
pub fn tr(text: &str) -> &str {
    if GERMAN.load(Ordering::Relaxed) { german(text).unwrap_or(text) } else { text }
}

/// German: (English, German). A two-line ribbon label keeps its line break. Commands use the
/// infinitive ("Einfügen", "Ersetzen"), states the adjective ("Fett", "Kursiv").
const DE: &[(&str, &str)] = &[
    ("File", "Datei"),
    ("Home", "Start"),
    ("Insert", "Einfügen"),
    ("Draw", "Zeichnen"),
    ("Design", "Entwurf"),
    ("Transitions", "Übergänge"),
    ("Animations", "Animationen"),
    ("Slide Show", "Bildschirmpräsentation"),
    ("Record", "Aufzeichnen"),
    ("Review", "Überprüfen"),
    ("View", "Ansicht"),
    ("Chart Design", "Diagrammentwurf"),
    ("Handout Master", "Handzettelmaster"),
    ("Layout", "Layout"),
    ("Notes Master", "Notizenmaster"),
    ("Picture Format", "Bildformat"),
    ("Playback", "Wiedergabe"),
    ("Shape Format", "Formformat"),
    ("Slide Master", "Folienmaster"),
    ("SmartArt Design", "SmartArt-Entwurf"),
    ("Table Design", "Tabellenentwurf"),
    ("Color", "Farbe"),
    ("Select All", "Alles auswählen"),
    ("Select None", "Auswahl aufheben"),
    ("Go to", "Gehe zu"),
    ("Ignore All", "Alle ignorieren"),
    ("Add to Dictionary", "Zum Wörterbuch hinzufügen"),
    ("Delete Repeated Word", "Wiederholtes Wort löschen"),
    ("Show", "Anzeigen"),
    ("Delete", "Löschen"),
    ("Clear", "Löschen"),
    ("Reject", "Ablehnen"),
    ("Accept", "Annehmen"),
    ("Picture", "Bild"),
    ("Columns…", "Spalten…"),
    ("Width", "Breite"),
    ("Preview", "Vorschau"),
    ("Effect\nOptions", "Effekt-\noptionen"),
    ("Add\nAnimation", "Animation\nhinzufügen"),
    ("Animation\nPane", "Animations-\nbereich"),
    ("Trigger", "Auslöser"),
    ("Animation\nPainter", "Animation\nübertragen"),
    ("Duration", "Dauer"),
    ("Delay", "Verzögerung"),
    ("Group", "Gruppieren"),
    ("Ungroup", "Gruppierung aufheben"),
    ("Regroup", "Gruppierung wiederherstellen"),
    ("Rotate Right 90°", "Rechtsdrehung 90°"),
    ("Rotate Left 90°", "Linksdrehung 90°"),
    ("Flip Vertical", "Vertikal spiegeln"),
    ("Flip Horizontal", "Horizontal spiegeln"),
    ("Distribute Horizontally", "Horizontal verteilen"),
    ("Distribute Vertically", "Vertikal verteilen"),
    ("Bring\nForward", "Ebene nach\nvorne"),
    ("Bring Forward", "Ebene nach vorne"),
    ("Bring to Front", "In den Vordergrund"),
    ("Send\nBackward", "Ebene nach\nhinten"),
    ("Send Backward", "Ebene nach hinten"),
    ("Send to Back", "In den Hintergrund"),
    ("Selection\nPane", "Auswahl-\nbereich"),
    ("Align", "Ausrichten"),
    ("Rotate", "Drehen"),
    ("Height", "Höhe"),
    ("Change\nChart Type", "Diagrammtyp\nändern"),
    ("Edit\nData", "Daten\nbearbeiten"),
    ("Browse for Themes…", "Nach Designs suchen…"),
    ("Colors", "Farben"),
    ("Fonts", "Schriftarten"),
    ("Effects", "Effekte"),
    ("Background\nStyles", "Hintergrund-\nformate"),
    ("Slide\nSize", "Folien-\ngröße"),
    ("Format\nBackground", "Hintergrund\nformatieren"),
    ("Design\nIdeas", "Design-\nideen"),
    ("Select", "Auswählen"),
    ("Eraser", "Radierer"),
    ("Ink to\nShape", "Freihand\nin Form"),
    ("Ink to\nText", "Freihand\nin Text"),
    ("Ink to\nMath", "Freihand\nin Formel"),
    ("Paste", "Einfügen"),
    ("Cut", "Ausschneiden"),
    ("Copy", "Kopieren"),
    ("Format", "Format"),
    ("New\nSlide", "Neue\nFolie"),
    ("Reset", "Zurücksetzen"),
    ("Section", "Abschnitt"),
    ("Increase Font Size", "Schrift vergrößern"),
    ("Decrease Font Size", "Schrift verkleinern"),
    ("Clear All Formatting", "Alle Formatierungen löschen"),
    ("Bold", "Fett"),
    ("Italic", "Kursiv"),
    ("Underline", "Unterstrichen"),
    ("Strikethrough", "Durchgestrichen"),
    ("Superscript", "Hochgestellt"),
    ("Subscript", "Tiefgestellt"),
    ("Character Spacing", "Zeichenabstand"),
    ("Change Case", "Groß-/Kleinschreibung"),
    ("Text Highlight Color", "Texthervorhebungsfarbe"),
    ("Font Color", "Schriftfarbe"),
    ("Decrease List Level", "Listenebene verringern"),
    ("Increase List Level", "Listenebene erhöhen"),
    ("Line Spacing", "Zeilenabstand"),
    ("Columns", "Spalten"),
    ("Align Left", "Linksbündig"),
    ("Center", "Zentriert"),
    ("Align Right", "Rechtsbündig"),
    ("Justify", "Blocksatz"),
    ("Text Direction", "Textrichtung"),
    ("Align Text", "Text ausrichten"),
    ("Convert to SmartArt", "In SmartArt umwandeln"),
    ("Shapes", "Formen"),
    ("Text Box", "Textfeld"),
    ("Arrange", "Anordnen"),
    ("Quick\nStyles", "Schnell-\nformate"),
    ("Shape Fill", "Fülleffekt"),
    ("Shape Outline", "Formkontur"),
    ("Shape Effects", "Formeffekte"),
    ("Find", "Suchen"),
    ("Replace", "Ersetzen"),
    ("Replace Fonts…", "Schriftarten ersetzen…"),
    ("Photo\nAlbum", "Foto-\nalbum"),
    ("New Photo Album…", "Neues Fotoalbum…"),
    ("Edit Photo Album…", "Fotoalbum bearbeiten…"),
    ("Table", "Tabelle"),
    ("Pictures", "Bilder"),
    ("Screenshot", "Screenshot"),
    ("Icons", "Symbole"),
    ("SmartArt", "SmartArt"),
    ("Chart", "Diagramm"),
    ("Zoom", "Zoom"),
    ("Link", "Link"),
    ("Action", "Aktion"),
    ("Comment", "Kommentar"),
    ("Text\nBox", "Text-\nfeld"),
    ("Header &\nFooter", "Kopf- und\nFußzeile"),
    ("WordArt", "WordArt"),
    ("Date &\nTime", "Datum und\nUhrzeit"),
    ("Slide\nNumber", "Folien-\nnummer"),
    ("Equation", "Formel"),
    ("Symbol", "Sonderzeichen"),
    ("Video", "Video"),
    ("Audio", "Audio"),
    ("Insert\nLayout", "Layout\neinfügen"),
    ("Rename", "Umbenennen"),
    ("Insert\nPlaceholder", "Platzhalter\neinfügen"),
    ("Themes", "Designs"),
    ("Close\nMaster View", "Masteransicht\nschließen"),
    ("Remove\nBackground", "Hintergrund\nentfernen"),
    ("Corrections", "Korrekturen"),
    ("Artistic\nEffects", "Künstlerische\nEffekte"),
    ("Transparency", "Transparenz"),
    ("Change Picture", "Bild ändern"),
    ("Reset Picture", "Bild zurücksetzen"),
    ("Compress Pictures", "Bilder komprimieren"),
    ("Picture Border", "Bildrahmen"),
    ("Reset Style", "Formatvorlage zurücksetzen"),
    ("Alt\nText", "Alternativ-\ntext"),
    ("Crop", "Zuschneiden"),
    ("Trim", "Kürzen"),
    ("Volume", "Lautstärke"),
    ("Customize Quick Access Toolbar", "Symbolleiste für den Schnellzugriff anpassen"),
    ("Spelling", "Rechtschreibung"),
    ("Thesaurus", "Thesaurus"),
    ("Check\nAccessibility", "Barrierefreiheit\nprüfen"),
    ("Translate", "Übersetzen"),
    ("Language", "Sprache"),
    ("Compare", "Vergleichen"),
    ("Accept\nAll", "Alle\nannehmen"),
    ("End\nReview", "Überprüfung\nbeenden"),
    ("New\nComment", "Neuer\nKommentar"),
    ("Show\nComments", "Kommentare\nanzeigen"),
    ("Edit\nShape", "Form\nbearbeiten"),
    ("Merge\nShapes", "Formen\nzusammenführen"),
    ("Shape\nStyles", "Form-\nformate"),
    ("Text Fill", "Textfüllung"),
    ("Text Outline", "Textkontur"),
    ("Text Effects", "Texteffekte"),
    ("Format\nPane", "Format-\nbereich"),
    ("Play from\nStart", "Von Beginn\nan"),
    ("Play from\nCurrent Slide", "Ab aktueller\nFolie"),
    ("Presenter\nView", "Referenten-\nansicht"),
    ("Custom\nShow", "Zielgruppen-\npräsentation"),
    ("Set Up\nSlide Show", "Präsentation\neinrichten"),
    ("Hide\nSlide", "Folie\nausblenden"),
    ("Rehearse\nTimings", "Anzeigedauern\ntesten"),
    ("Cameo", "Cameo"),
    ("From\nBeginning", "Von\nBeginn an"),
    ("From\nCurrent Slide", "Ab aktueller\nFolie"),
    ("Learn\nMore", "Weitere\nInfos"),
    ("Text\nPane", "Text-\nbereich"),
    ("Add Shape", "Form hinzufügen"),
    ("Promote", "Höher stufen"),
    ("Demote", "Tiefer stufen"),
    ("Insert Picture", "Bild einfügen"),
    ("Layouts", "Layouts"),
    ("Convert", "Umwandeln"),
    ("Change Colors", "Farben ändern"),
    ("SmartArt Styles", "SmartArt-Formate"),
    ("Shading", "Schattierung"),
    ("Borders", "Rahmen"),
    ("Delete Columns", "Spalten löschen"),
    ("Delete Rows", "Zeilen löschen"),
    ("Delete Table", "Tabelle löschen"),
    ("Insert\nAbove", "Darüber\neinfügen"),
    ("Insert\nBelow", "Darunter\neinfügen"),
    ("Insert\nLeft", "Links\neinfügen"),
    ("Insert\nRight", "Rechts\neinfügen"),
    ("Merge\nCells", "Zellen\nverbinden"),
    ("Split\nCells", "Zellen\nteilen"),
    ("Distribute Rows", "Zeilen verteilen"),
    ("Distribute Columns", "Spalten verteilen"),
    ("Apply\nTo All", "Auf alle\nanwenden"),
    ("After", "Nach"),
    ("Slide\nMaster", "Folien-\nmaster"),
    ("Handout\nMaster", "Handzettel-\nmaster"),
    ("Notes\nMaster", "Notizen-\nmaster"),
    ("Grid\nOptions", "Raster-\noptionen"),
    ("Notes", "Notizen"),
    ("Fit to\nWindow", "An Fenster\nanpassen"),
    ("New\nWindow", "Neues\nFenster"),
    ("Grayscale", "Graustufen"),
    ("Dark\nMode", "Dunkler\nModus"),
    ("File (Alt+F)", "Datei (Alt+F)"),
    ("Options", "Optionen"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn german_rows_are_unique_and_keep_their_line_breaks() {
        let mut seen = std::collections::HashSet::new();
        for (english, german) in DE {
            assert!(seen.insert(*english), "duplicate row {english:?}");
            assert_eq!(english.matches('\n').count(), german.matches('\n').count(), "{english:?}");
            assert_eq!(english.ends_with('…'), german.ends_with('…'), "{english:?}");
        }
    }

    // The language itself is process-wide and other tests read English labels, so these tests
    // check the pure parts and never switch it.
    #[test]
    fn only_german_locales_are_german() {
        for locale in ["de", "de-AT", "DE-ch", "de_CH.UTF-8"] {
            assert!(is_german(locale), "{locale}");
        }
        for locale in ["en-US", "fr", "", "den"] {
            assert!(!is_german(locale), "{locale}");
        }
    }

    #[test]
    fn rows_are_looked_up_by_their_english_text() {
        assert_eq!(german("New\nSlide"), Some("Neue\nFolie"));
        assert_eq!(german("SmartArt"), Some("SmartArt"));
        assert_eq!(german("No such label"), None);
    }
}
