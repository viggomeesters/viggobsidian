use wiki_layout_core::headings::anchor_count;

#[test]
fn nested_source_quotes_keep_anchors_and_exclude_quoted_examples() {
    let source = "> [!quote] Original\n> > ## Observed state\n> > Evidence.\n> > ```md\n> > ## Example\n> > ```\n> > <!--\n> > ## Hidden\n> > -->\n";
    assert_eq!(anchor_count(source, "Observed state"), 1);
    for absent in ["Example", "Hidden", "Missing"] {
        assert_eq!(anchor_count(source, absent), 0);
    }
    assert_eq!(
        anchor_count(&format!("{source}## Observed state\n"), "Observed state"),
        2
    );
}

#[test]
fn quoted_separators_are_not_document_frontmatter() {
    let source = "> ---\n> ## Visible\n> Text ^visible\n> ---\n";
    for document in [
        source.to_owned(),
        format!("---\n# YAML\nvalue: hidden ^hidden\n---\n{source}"),
    ] {
        assert_eq!(anchor_count(&document, "Visible"), 1);
        assert_eq!(anchor_count(&document, "^visible"), 1);
        assert_eq!(anchor_count(&document, "YAML"), 0);
        assert_eq!(anchor_count(&document, "^hidden"), 0);
    }
}
