//! Shared Project Koppelen wire format and validation for caller-owned catalogs.
use serde_json::{Value, json};

#[derive(Clone, Copy)]
pub enum ChoiceKind {
    Work,
    Private,
}

impl ChoiceKind {
    fn wrapper(self) -> &'static str {
        match self {
            Self::Work => "workProjectChoice",
            Self::Private => "privateProjectChoice",
        }
    }

    fn presentation_field(self) -> &'static str {
        match self {
            Self::Work => "presentation_link",
            Self::Private => "presentation_links",
        }
    }
}

/// The caller computes the hashes from its own source and catalog.
pub struct Binding<'a> {
    pub path: &'a str,
    pub source_hash: &'a str,
    pub catalog_hash: &'a str,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ChoiceError {
    InvalidJson,
    Stale,
    MissingChoice,
}

impl Binding<'_> {
    pub fn item(
        &self,
        kind: ChoiceKind,
        entry: &Value,
        subtitle: &Value,
        presentation: Option<Value>,
    ) -> Value {
        let mut choice = json!({
            "path": self.path,
            "source_hash": self.source_hash,
            "catalog_hash": self.catalog_hash,
            "id": entry["id"],
        });
        if let Some(presentation) = presentation {
            choice[kind.presentation_field()] = presentation;
        }
        json!({
            "id": entry["id"],
            "title": entry["title"],
            "subtitle": subtitle,
            "value": json!({ (kind.wrapper()): choice }).to_string(),
        })
    }

    /// Resolves the exact selected ID. The caller retains the original input for receipt replay.
    pub fn resolve<'a>(
        &self,
        kind: ChoiceKind,
        input: &str,
        entries: &'a [Value],
        required_root: Option<&str>,
    ) -> Result<(&'a Value, Value), ChoiceError> {
        let mut value: Value = serde_json::from_str(input).map_err(|_| ChoiceError::InvalidJson)?;
        let choice = value
            .get_mut(kind.wrapper())
            .map(Value::take)
            .unwrap_or_default();
        if choice["path"] != self.path
            || choice["source_hash"] != self.source_hash
            || choice["catalog_hash"] != self.catalog_hash
            || required_root.is_some_and(|root| choice["active_root"] != root)
        {
            return Err(ChoiceError::Stale);
        }
        let entry = entries
            .iter()
            .find(|entry| entry["id"] == choice["id"])
            .ok_or(ChoiceError::MissingChoice)?;
        Ok((entry, choice))
    }
}

pub fn envelope(items: Vec<Value>) -> Value {
    json!({
        "ok": true,
        "choices": {
            "title": "Project Koppelen",
            "placeholder": "Zoek project of onderdeel…",
            "items": items,
        },
    })
}

pub fn is_component(entry: &Value) -> bool {
    entry["id"] != entry["project"]
}

pub fn full_path_link(path: &str) -> String {
    format!("[[{}]]", path.trim_end_matches(".md"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const BINDING: Binding<'static> = Binding {
        path: "inbox/example.md",
        source_hash: "source-fixture",
        catalog_hash: "catalog-fixture",
    };

    #[test]
    fn keeps_the_wire_bytes_and_optional_presentation() {
        let entry = json!({"id":"part","title":"Voorbeeld 🧪","project":"parent"});
        let work = BINDING.item(
            ChoiceKind::Work,
            &entry,
            &json!("P-01 · Voorbeeld · 2026-11"),
            Some(Value::Null),
        );
        assert_eq!(
            work["value"],
            r#"{"workProjectChoice":{"catalog_hash":"catalog-fixture","id":"part","path":"inbox/example.md","presentation_link":null,"source_hash":"source-fixture"}}"#
        );

        let private = BINDING.item(ChoiceKind::Private, &entry, &json!("P-01"), None);
        assert_eq!(
            private["value"],
            r#"{"privateProjectChoice":{"catalog_hash":"catalog-fixture","id":"part","path":"inbox/example.md","source_hash":"source-fixture"}}"#
        );
        assert_eq!(private["title"], "Voorbeeld 🧪");
        assert_eq!(
            envelope(vec![work, private])["choices"]["items"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        assert!(is_component(&entry));
        assert_eq!(full_path_link("fictional/p/p.md.md"), "[[fictional/p/p]]");
    }

    #[test]
    fn validates_the_source_and_scope_before_selecting_the_exact_id() {
        let entries = [json!({"id":"part","project":"parent"})];
        let mut value = json!({"privateProjectChoice":{
            "path":BINDING.path,"source_hash":BINDING.source_hash,
            "catalog_hash":BINDING.catalog_hash,"id":"part","active_root":"/fixture",
            "presentation_links":{"project":{"target":"fictional/project"}},
        }});
        let resolve =
            |input: &str| BINDING.resolve(ChoiceKind::Private, input, &entries, Some("/fixture"));
        assert_eq!(resolve("{"), Err(ChoiceError::InvalidJson));
        for serialized in ["null", "[]", "{\"privateProjectChoice\":null}"] {
            assert_eq!(resolve(serialized), Err(ChoiceError::Stale));
        }
        assert_eq!(
            resolve(&value.to_string()),
            Ok((&entries[0], value["privateProjectChoice"].clone()))
        );
        value["privateProjectChoice"]["id"] = json!("missing");
        assert_eq!(resolve(&value.to_string()), Err(ChoiceError::MissingChoice));
        value["privateProjectChoice"]["active_root"] = json!("/other");
        assert_eq!(resolve(&value.to_string()), Err(ChoiceError::Stale));
    }
}
