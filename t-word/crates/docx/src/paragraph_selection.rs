//! A checked correspondence between Word's main story and the saved OOXML paragraphs.
//! Word can merge or expand imported blocks. Only a direct, unambiguous paragraph
//! sequence can use the ordinal understood by the NAS file reader.
use crate::{
    package::{Package, rt},
    xml::El,
};
use std::{collections::HashSet, io::Cursor};
use wordcraft_doc::{Block, Document};

#[derive(Debug)]
pub struct SavedParagraphs(Vec<String>);

impl SavedParagraphs {
    pub fn from_docx(bytes: &[u8]) -> Option<Self> {
        if bytes.len() > 32 * 1024 * 1024 || !unique_package(bytes) {
            return None;
        }
        let pkg = Package::open_limited(bytes, 8 << 20, 32 << 20).ok()?;
        let rels = pkg.rels("");
        if rels.by_type(rt::OFFICE_DOC).is_some_and(|rel| rel.external || rel.target != "word/document.xml") {
            return None;
        }
        // The server numbers paragraphs in the transitional WordprocessingML
        // namespace. The display importer also accepts Strict OOXML.
        let root = pkg.xml_strict("word/document.xml").ok()??;
        if !transitional_text(pkg.get("word/document.xml")?) {
            return None;
        }
        Self::from_root(&root)
    }

    fn from_root(root: &El) -> Option<Self> {
        if root.name != "w:document" {
            return None;
        }
        let mut roots = root.els();
        let body = roots.next()?;
        if body.name != "w:body" || roots.next().is_some() {
            return None;
        }
        let mut paragraphs = Vec::new();
        for block in body.els() {
            if block.name == "w:sectPr" && safe_properties(block) {
                continue;
            }
            if block.name != "w:p" || paragraphs.len() >= 65536 {
                return None;
            }
            let mut text = String::new();
            for run in block.els() {
                match run.name.as_str() {
                    "w:pPr" if safe_properties(run) => {}
                    "w:r" => {
                        for item in run.els() {
                            match item.name.as_str() {
                                "w:rPr" if safe_properties(item) => {}
                                "w:t" if item.els().next().is_none() => text.push_str(&item.text()),
                                _ => return None,
                            }
                        }
                    }
                    _ => return None,
                }
            }
            paragraphs.push(text);
        }
        (!paragraphs.is_empty()).then_some(Self(paragraphs))
    }

    pub fn matches(&self, doc: &Document) -> bool {
        self.0.len() == doc.body.len()
            && self.0.iter().zip(doc.body.iter()).all(|(source, block)| matches!(&**block, Block::Para(para) if para.text == *source))
    }
}

// Make sure the native and NAS readers cannot pick different entries. Some
// ZIP readers deduplicate identical names; compare their count to the archive.
fn unique_package(bytes: &[u8]) -> bool {
    if wordcraft_doc::zip_guard::check(bytes, crate::package::MAX_ENTRIES as u64).is_err() {
        return false;
    }
    let Ok(archive) = zip::ZipArchive::new(Cursor::new(bytes)) else {
        return false;
    };
    if wordcraft_doc::zip_guard::check(bytes, archive.len() as u64).is_err() {
        return false;
    }
    let mut names = HashSet::new();
    let mut main = false;
    for name in archive.file_names() {
        if name.starts_with('/') || !names.insert(name.to_ascii_lowercase()) {
            return false;
        }
        main |= name == "word/document.xml";
    }
    main
}

fn transitional_text(bytes: &[u8]) -> bool {
    use quick_xml::{events::Event, name::ResolveResult, reader::NsReader};
    let mut reader = NsReader::from_reader(bytes);
    loop {
        match reader.read_resolved_event() {
            Ok((ns, Event::Start(el) | Event::Empty(el))) if matches!(el.local_name().as_ref(), b"p" | b"t" | b"document") => {
                if !matches!(ns, ResolveResult::Bound(uri) if uri.as_ref() == b"http://schemas.openxmlformats.org/wordprocessingml/2006/main") {
                    return false;
                }
            }
            Ok((_, Event::Eof)) => return true,
            Ok((_, Event::DocType(_))) | Err(_) => return false,
            _ => {}
        }
    }
}

// Property elements may contain further formatting. Any nested paragraph, frame,
// alternate content or embedded text story would invalidate a positional mapping.
fn safe_properties(root: &El) -> bool {
    root.els().all(|el| {
        el.name.starts_with("w:")
            && !matches!(el.name.as_str(), "w:p" | "w:framePr" | "w:altChunk" | "w:txbxContent" | "w:r" | "w:t" | "w:sdt" | "w:tbl")
            && safe_properties(el)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn strict_and_rebound_paragraph_namespaces_are_not_nas_ordinals() {
        let w = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
        let xml = |body: &str| format!(r#"<w:document xmlns:w="{w}"><w:body>{body}</w:body></w:document>"#);
        assert!(transitional_text(xml("<w:p><w:r><w:t>Text</w:t></w:r></w:p>").as_bytes()));
        assert!(!transitional_text(xml(r#"<w:p xmlns:w="urn:unrelated"/>"#).as_bytes()));
        assert!(!transitional_text(xml("<w:p/>").replace(w, "http://purl.oclc.org/ooxml/wordprocessingml/main").as_bytes()));
    }

    #[test]
    fn ordinary_and_empty_paragraphs_have_stable_ordinals() {
        let root = |body: &str| {
            crate::xml::parse_strict(
                format!(r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>{body}</w:body></w:document>"#)
                    .as_bytes(),
            )
            .unwrap()
        };
        let normal = "<w:p><w:r><w:t>Gleich</w:t></w:r></w:p>";
        let map = SavedParagraphs::from_root(&root(&format!("{normal}<w:p/>{normal}<w:sectPr/>"))).unwrap();
        assert_eq!(map.0, ["Gleich", "", "Gleich"]);
        for special in
            ["<w:tbl/>", "<w:altChunk/>", "<w:p><w:pPr><w:framePr/></w:pPr></w:p>", "<w:p><w:r><w:txbxContent><w:p/></w:txbxContent></w:r></w:p>"]
        {
            assert!(SavedParagraphs::from_root(&root(&format!("{special}{normal}"))).is_none());
        }
    }
}
