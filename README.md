# TitanOS-Office

Die Office- und Kreativprogramme von [TitanOS](https://github.com/ra5on/TitanOS): **T-Word** (Texte), **T-PDF** (PDF), **T-Exel** (Tabellen), **T-Deck** (Präsentationen), **T-CAD** (CAD-Zeichnungen), **T-Fig** (Interface-Design), **T-Film** (Videoschnitt), **T-Light** (Fotobibliothek und Raw-Entwicklung), **T-Vector** (Vektorgrafik), **T-Design** (Layout und Satz), **T-Effect** (Motion Graphics und Effekte), **T-Sound** (Audio), **T-Texture** (3D-Texturen), **T-Material** (prozedurale Materialien).

Sie laufen als Web-Fassung (WebAssembly) direkt vom NAS. TitanOS liefert sie unter seiner eigenen Adresse aus. Deshalb funktionieren sie auch hinter Cloudflare Access, ohne Zusatzregel. Dateien öffnen und speichern sie direkt in den Freigaben des NAS.

## Herkunft

| Programm | beruht auf | Stand |
|---|---|---|
| T-Word (`t-word/`) | [WordCraft](https://github.com/storytold/wordcraft) | v0.16.0 (02a802cb) |
| T-PDF (`t-pdf/`) | [PdfCraft](https://github.com/storytold/pdfcraft) | v0.19.0 (8c572792) |
| T-Exel (`t-exel/`) | [GridCraft](https://github.com/storytold/gridcraft) | v0.17.0 (a6a6a49a) |
| T-Deck (`t-deck/`) | [DeckCraft](https://github.com/storytold/deckcraft) | v0.16.0 (a3e2254d) |
| T-CAD (`t-cad/`) | [CADCraft](https://github.com/storytold/cadcraft) | v0.18.0 (32238e83) |
| T-Fig (`t-fig/`) | [FigCraft](https://github.com/storytold/figcraft) | v0.16.0 (f45b2616) |
| T-Film (`t-film/`) | [FilmCraft](https://github.com/storytold/filmcraft) | v0.18.0 (015d5b3e) |
| T-Light (`t-light/`) | [LightCraft](https://github.com/storytold/lightcraft) | v0.19.0 (16636c45) |
| T-Vector (`t-vector/`) | [VectorCraft](https://github.com/storytold/vectorcraft) | v0.22.0 (654749c1) |
| T-Design (`t-design/`) | [DesignCraft](https://github.com/storytold/designcraft) | v0.18.0 (58ac6d29) |
| T-Effect (`t-effect/`) | [EffectCraft](https://github.com/storytold/effectcraft) | v0.22.0 (42e3c6a6) |
| T-Sound (`t-sound/`) | [SoundCraft](https://github.com/storytold/soundcraft) | v0.18.0 (5ff012ab) |
| T-Texture (`t-texture/`) | [TextureCraft](https://github.com/storytold/texturecraft) | v0.4.0 (16e9d215) |
| T-Material (`t-material/`) | [MaterialCraft](https://github.com/storytold/materialcraft) | v0.2.0 (a1bbd4a8) |

Die Originale stammen vom ArtCraft Team und stehen unter **MIT OR Apache-2.0**. Diese Fassungen stehen unter derselben Lizenz. Die Lizenztexte und Hinweise der Originale liegen in jedem Ordner, siehe auch [NOTICE](NOTICE).

**Was sich gegenüber den Originalen ändert:**
- **Keine ArtCraft-Marke.** Name, Schriftzug und Logos sind entfernt, wie es die Markenregel der Originale für Forks verlangt (`docs/brand/LICENSE-brand.txt` dort). Statt der Community-Links von ArtCraft führen die Links hierher.
- **Eigene Namen** in der Oberfläche: T-Word, T-PDF, T-Exel, T-Deck, T-CAD, T-Fig, T-Film, T-Light, T-Vector, T-Design, T-Effect, T-Sound, T-Texture, T-Material. Kennungen, Dateiformate und Crate-Namen der Originale bleiben.
- **Keine Abhängigkeit vom Update-Takt von storytold.** Die Bausteine, die die Originale aus storytolds Forks laden, kommen aus eigenen Kopien mit denselben Commits:
  - [TitanOS-craft-libs](https://github.com/ra5on/TitanOS-craft-libs)
  - [TitanOS-egui](https://github.com/ra5on/TitanOS-egui)
  - [TitanOS-winit](https://github.com/ra5on/TitanOS-winit)
  - [TitanOS-winprint](https://github.com/ra5on/TitanOS-winprint)

  Zwei weitere Bausteine liegen direkt im Repo: T-Effect nutzt die FilmCraft-Codecs aus `t-effect/vendor/filmcraft/` (storytold/filmcraft d5ead4ba), T-Design den INDD-Leser aus `t-design/vendor/indd/` (storytold/indd-utils 7f3060e5), beide MIT OR Apache-2.0.

  Neuerungen der Originale übernehmen wir gezielt, nicht automatisch.
- **Übersetzungen:** Die deutschen Kataloge liegen im Repo (`<programm>/i18n/`), übernommen aus [craft-i18n](https://github.com/storytold/craft-i18n) am Commit in `craft-i18n.rev`, mit den neuen Programmnamen. Beim Bauen wird nichts von storytold geladen.

## Bauen und Releases

Die CI (`.github/workflows/build.yml`) baut alle Web-Fassungen: die meisten mit Trunk, T-Film, T-Light und T-Effect wie ihre Originale mit `cargo xtask web` (wasm-bindgen). Die Schriften aus craft-fonts (bei T-Fig, T-Light und T-Vector im Original optional eingebettet) werden nicht geladen; diese Programme nutzen ihre eingebauten Schriften.

**Tag-Schema:** ein gemeinsamer Tag `vX.Y.Z` für alle Programme. Das Release enthält:
- `t-word-web-X.Y.Z.zip`
- `t-pdf-web-X.Y.Z.zip`
- `t-exel-web-X.Y.Z.zip`
- `t-deck-web-X.Y.Z.zip`
- `t-cad-web-X.Y.Z.zip`
- `t-fig-web-X.Y.Z.zip`
- `t-film-web-X.Y.Z.zip`
- `t-light-web-X.Y.Z.zip`
- `t-vector-web-X.Y.Z.zip`
- `t-design-web-X.Y.Z.zip`
- `t-effect-web-X.Y.Z.zip`
- `t-sound-web-X.Y.Z.zip`
- `t-texture-web-X.Y.Z.zip`
- `t-material-web-X.Y.Z.zip`
- `SHA256SUMS.txt`

Jedes Archiv enthält genau einen Ordner `t-<app>-web-X.Y.Z/`, in dessen oberster Ebene `index.html` liegt. Diesen Aufbau erwartet TitanOS (`backend/internal/webapps`). Im Katalog von TitanOS (`webapps/catalog.json`) stehen die Programme mit `repo: ra5on/TitanOS-Office`, der Version und dem SHA-256 des Archivs.

Lokal bauen braucht Rust (stable) mit dem Ziel `wasm32-unknown-unknown` und Trunk:

```bash
cd t-pdf/apps/pdfcraft-web && trunk build --release
```

## Weiterentwicklung

Als Nächstes kommt die direkte Anbindung an TitanOS (PLAN 75-18): Öffnen und Speichern über die Datei-API, die Dateiauswahl von TitanOS statt des Browser-Dialogs, Speichern ohne Download und zuletzt geöffnete Dateien. Bis dahin gelten die Anbindung und das Verhalten der Originale.
