//! Maintenance guard for the four feature suites expanded on 2026-10-16.
//! This checks navigation/annotation integrity, not semantic coverage or conformance.
use std::collections::BTreeSet;

const INDEX: &str = include_str!("../docs/REQUIREMENTS.md");
const SUITES: [(&str, &str); 4] = [
    ("arbitrary_writes", include_str!("arbitrary_writes.rs")),
    ("clock_set", include_str!("clock_set.rs")),
    ("transactions", include_str!("transactions.rs")),
    (
        "configurable_tables",
        include_str!("configurable_tables.rs"),
    ),
];

fn ids(text: &str) -> BTreeSet<&str> {
    text.split(|c: char| !(c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-'))
        .filter(|word| (word.starts_with("REQ-") || word.starts_with("POL-")) && word.len() > 4)
        .collect()
}

fn tests(source: &str) -> Vec<(&str, &str)> {
    // rustfmt keeps these top-level integration-test closing braces at column 0.
    // Only actual test attributes are admitted; helpers cannot supply annotations.
    let mut result = Vec::new();
    let mut rest = source;
    while let Some(pos) = rest.find("#[") {
        rest = &rest[pos + 2..];
        let (attribute, tail) = rest.split_once(']').expect("closed attribute");
        rest = tail;
        if !matches!(attribute, "test" | "tokio::test") {
            continue;
        }
        let (_, function) = rest.split_once("fn ").expect("test function");
        let (name, _) = function.split_once('(').expect("test name");
        let end = function.find("\n}").expect("top-level test closing brace") + 2;
        result.push((name, &function[..end]));
        rest = &function[end..];
    }
    result
}

#[test]
fn session_feature_tests_have_indexed_requirement_annotations() {
    // POL-REQUIREMENT-TRACEABILITY: every test in the scoped feature suites has
    // executable assertions and stable IDs linked to that exact test in the index.
    for (suite, source) in SUITES {
        let functions = tests(source);
        assert!(!functions.is_empty(), "no tests found in {suite}");
        for (name, body) in functions {
            let annotations = ids(body);
            assert!(
                !annotations.is_empty(),
                "{suite}::{name} lacks requirement IDs"
            );
            assert!(body.contains("assert"), "{suite}::{name} lacks assertions");
            for id in annotations {
                let link = format!("[{name}](../tests/{suite}.rs)");
                assert!(
                    INDEX
                        .lines()
                        .filter(|row| row.starts_with("| "))
                        .any(|row| {
                            let stable_ids = row.split('|').nth(1).unwrap();
                            ids(stable_ids).contains(id) && row.contains(&link)
                        }),
                    "{suite}::{name}: {id} needs an index row linking this exact test"
                );
            }
        }
    }
}

#[test]
fn session_requirement_links_name_existing_tests_and_matching_annotations() {
    // POL-REQUIREMENT-TRACEABILITY: index links into these suites must name
    // existing tests, and each indexed ID must occur in a linked test body.
    for row in INDEX.lines().filter(|row| row.starts_with("| ")) {
        let row_ids = ids(row.split('|').nth(1).unwrap());
        let mut linked_ids = BTreeSet::new();
        let mut scoped = false;
        for (suite, source) in SUITES {
            let suffix = format!("](../tests/{suite}.rs)");
            let mut remaining = row;
            while let Some(pos) = remaining.find(&suffix) {
                scoped = true;
                let before = &remaining[..pos];
                let name = before.rsplit_once('[').expect("link label").1;
                let functions = tests(source);
                let (_, body) = functions
                    .iter()
                    .find(|(test, _)| *test == name)
                    .unwrap_or_else(|| panic!("index names missing test {suite}::{name}"));
                linked_ids.extend(ids(body));
                remaining = &remaining[pos + suffix.len()..];
            }
        }
        if scoped {
            for id in row_ids {
                assert!(
                    linked_ids.contains(id),
                    "index row lacks a linked assertion annotation for {id}"
                );
            }
        }
    }
}
