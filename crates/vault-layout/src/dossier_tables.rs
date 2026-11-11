use serde_json::Value;

pub(crate) struct Column {
    property: &'static str,
    label: &'static str,
    width: u64,
}

const NOTE: Column = Column {
    property: "formula.filename",
    label: "Note",
    width: 360,
};
const SOURCE: &[Column] = &[
    Column {
        property: "formula.wiki_badge",
        label: "Status",
        width: 132,
    },
    Column {
        property: "formula.created_day",
        label: "Created",
        width: 110,
    },
    NOTE,
];
const TASK: &[Column] = &[
    Column {
        property: "note.status",
        label: "Status",
        width: 132,
    },
    Column {
        property: "formula.action_day",
        label: "Action date",
        width: 110,
    },
    NOTE,
];
pub const RAW_BASE: &str = "99-system/vault/vault-navigation/standaard/wiki/vault-raw.base";
pub(crate) type ViewContract = (&'static str, &'static [Column]);
pub const CONTRACTS: &[(&str, &[ViewContract])] = &[
    (RAW_BASE, &[("Latest sources", SOURCE)]),
    (
        "99-system/vault/vault-navigation/standaard/wiki/vault-dossier.base",
        &[("Open tasks", TASK)],
    ),
];

pub fn findings(source: &str, expected: &[ViewContract]) -> Vec<String> {
    let options = serde_saphyr::options! {
        duplicate_keys: serde_saphyr::DuplicateKeyPolicy::Error,
    };
    let Ok(base) = serde_saphyr::from_str_with_options::<Value>(source, options) else {
        return vec!["expected a valid Base YAML mapping with unique keys".into()];
    };
    let mut findings = Vec::new();
    if base["formulas"]["filename"].as_str() != Some("link(file.path, file.name)") {
        findings.push("set formula.filename to link(file.path, file.name)".into());
    }
    for (name, columns) in expected {
        let mut views = base["views"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|view| view["name"].as_str() == Some(name));
        let Some(view) = views
            .next()
            .filter(|view| view["type"].as_str() == Some("table") && views.next().is_none())
        else {
            findings.push(format!("{name}: expected exactly one table view"));
            continue;
        };
        if view["limit"].as_u64() != Some(25) {
            findings.push(format!("{name}: set the row limit to 25"));
        }
        if !view["order"].as_array().is_some_and(|order| {
            order.len() == columns.len()
                && order.iter().zip(*columns).all(|(value, column)| {
                    value
                        .as_str()
                        .is_some_and(|key| canonical(key) == canonical(column.property))
                })
        }) {
            let order = columns
                .iter()
                .map(|column| column.property)
                .collect::<Vec<_>>()
                .join(", ");
            findings.push(format!("{name}: expected column order {order}"));
        }
        for column in *columns {
            let key = column.property;
            if canonical(key) != key
                && [&base["properties"], &view["columnSize"]]
                    .iter()
                    .any(|mapping| {
                        mapping.get(key).is_some() && mapping.get(canonical(key)).is_some()
                    })
            {
                findings.push(format!("{name}: use only one spelling of {key}"));
                continue;
            }
            if property(&base["properties"], column.property)
                .is_none_or(|value| value["displayName"].as_str() != Some(column.label))
            {
                findings.push(format!(
                    "{name}: label {} as {}",
                    column.property, column.label
                ));
            }
            if property(&view["columnSize"], column.property).and_then(Value::as_u64)
                != Some(column.width)
            {
                findings.push(format!(
                    "{name}: set {} width to {} px",
                    column.label, column.width
                ));
            }
        }
    }
    findings
}

// Obsidian can save a note property's key with or without the note. prefix.
fn canonical(key: &str) -> &str {
    match key {
        "note.status" => "status",
        _ => key,
    }
}

fn property<'a>(mapping: &'a Value, key: &str) -> Option<&'a Value> {
    mapping.get(key).or_else(|| mapping.get(canonical(key)))
}

#[cfg(test)]
mod tests {
    use super::*;

    const RAW: &str = include_str!("../fixtures/vault-raw.base");
    const DOSSIER: &str = include_str!("../fixtures/vault-dossier.base");

    #[test]
    fn shared_views_follow_the_filename_and_alignment_contract() {
        assert!(findings(RAW, CONTRACTS[0].1).is_empty());
        assert!(findings(DOSSIER, CONTRACTS[1].1).is_empty());
        let unlimited = DOSSIER.replace("    limit: 25\n", "");
        assert!(
            findings(&unlimited, CONTRACTS[1].1)
                .iter()
                .any(|msg| msg.contains("row limit"))
        );

        let renamed = DOSSIER.replace("- formula.filename", "- formula.title");
        assert!(
            findings(&renamed, CONTRACTS[1].1)
                .iter()
                .any(|msg| msg.contains("column order"))
        );

        let reordered = RAW.replace(
            "      - formula.wiki_badge\n      - formula.created_day",
            "      - formula.created_day\n      - formula.wiki_badge",
        );
        assert!(
            findings(&reordered, CONTRACTS[0].1)
                .iter()
                .any(|msg| msg.contains("column order"))
        );

        let substituted_title =
            DOSSIER.replace("link(file.path, file.name)", "link(file.path, note.title)");
        assert!(
            findings(&substituted_title, CONTRACTS[1].1)
                .iter()
                .any(|msg| msg.contains("set formula.filename"))
        );

        let resized = DOSSIER.replace("formula.filename: 360", "formula.filename: 359");
        assert!(
            findings(&resized, CONTRACTS[1].1)
                .iter()
                .any(|msg| msg.contains("Note width"))
        );

        let native_alias = DOSSIER.replace("note.status", "status");
        assert!(findings(&native_alias, CONTRACTS[1].1).is_empty());

        let conflicting_alias = DOSSIER.replace(
            "      note.status: 132",
            "      note.status: 132\n      status: 90",
        );
        assert!(
            findings(&conflicting_alias, CONTRACTS[1].1)
                .iter()
                .any(|msg| msg.contains("only one spelling"))
        );
    }
}
