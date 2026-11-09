# TitanOS-Office

Die Office-Programme von [TitanOS](https://github.com/ra5on/TitanOS): **T-Word** (Texte), **T-PDF** (PDF) und **T-Exel** (Tabellen).

Sie laufen als Web-Fassung (WebAssembly) direkt vom NAS. TitanOS liefert sie unter seiner eigenen Adresse aus. Deshalb funktionieren sie auch hinter Cloudflare Access, ohne Zusatzregel. Dateien öffnen und speichern sie direkt in den Freigaben des NAS.

## Herkunft

| Programm | beruht auf | Stand |
|---|---|---|
| T-Word (`t-word/`) | [WordCraft](https://github.com/storytold/wordcraft) | v0.16.0 (02a802cb) |
| T-PDF (`t-pdf/`) | [PdfCraft](https://github.com/storytold/pdfcraft) | v0.19.0 (8c572792) |
| T-Exel (`t-exel/`) | [GridCraft](https://github.com/storytold/gridcraft) | v0.17.0 (a6a6a49a) |

Die Originale stammen vom ArtCraft Team und stehen unter **MIT OR Apache-2.0**. Diese Fassungen stehen unter derselben Lizenz. Die Lizenztexte und Hinweise der Originale liegen in jedem Ordner, siehe auch [NOTICE](NOTICE).

**Was sich gegenüber den Originalen ändert:**
- **Keine ArtCraft-Marke.** Name, Schriftzug und Logos sind entfernt, wie es die Markenregel der Originale für Forks verlangt (`docs/brand/LICENSE-brand.txt` dort). Statt der Community-Links von ArtCraft führen die Links hierher.
- **Eigene Namen** in der Oberfläche: T-Word, T-PDF, T-Exel.
- **Keine Abhängigkeit vom Update-Takt von storytold.** Die Bausteine, die die Originale aus storytolds Forks laden, kommen aus eigenen Kopien mit denselben Commits:
  - [TitanOS-craft-libs](https://github.com/ra5on/TitanOS-craft-libs)
  - [TitanOS-egui](https://github.com/ra5on/TitanOS-egui)
  - [TitanOS-winit](https://github.com/ra5on/TitanOS-winit)
  - [TitanOS-winprint](https://github.com/ra5on/TitanOS-winprint)

  Neuerungen der Originale übernehmen wir gezielt, nicht automatisch.
- **Übersetzungen** (darunter Deutsch) kommen beim Bauen aus [craft-i18n](https://github.com/storytold/craft-i18n), fest auf den Commit in `craft-i18n.rev`.

## Bauen und Releases

Die CI (`.github/workflows/build.yml`) baut die drei Web-Fassungen mit Trunk.

**Tag-Schema:** ein gemeinsamer Tag `vX.Y.Z` für alle drei Programme. Das Release enthält:
- `t-word-web-X.Y.Z.zip`
- `t-pdf-web-X.Y.Z.zip`
- `t-exel-web-X.Y.Z.zip`
- `SHA256SUMS.txt`

Jedes Archiv enthält genau einen Ordner `t-<app>-web-X.Y.Z/`, in dessen oberster Ebene `index.html` liegt. Diesen Aufbau erwartet TitanOS (`backend/internal/webapps`). Im Katalog von TitanOS (`webapps/catalog.json`) stehen die Programme mit `repo: ra5on/TitanOS-Office`, der Version und dem SHA-256 des Archivs.

Lokal bauen braucht Rust (stable) mit dem Ziel `wasm32-unknown-unknown` und Trunk:

```bash
cd t-pdf/apps/pdfcraft-web && trunk build --release
```

## Weiterentwicklung

Als Nächstes kommt die direkte Anbindung an TitanOS (PLAN 75-18): Öffnen und Speichern über die Datei-API, die Dateiauswahl von TitanOS statt des Browser-Dialogs, Speichern ohne Download und zuletzt geöffnete Dateien. Bis dahin gelten die Anbindung und das Verhalten der Originale.
