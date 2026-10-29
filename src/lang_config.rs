use include_dir::{Dir, include_dir};
use serde::Deserialize;
use std::path::Path;

static LANG_DIR: Dir<'_> = include_dir!("$STAGED_LANGS_DIR");

/// Per-language analyzer configuration.
///
/// Every field is optional: a missing key in `config.yaml` deserializes to `None` (serde
/// special-cases `Option<T>`). The analyzer treats `None` and an empty value identically — it
/// simply skips the corresponding operation. So a config only needs to declare the capabilities
/// the language actually uses.
#[derive(Clone, Deserialize, Debug, Default)]
pub struct LangConfig {
    pub file_extensions: Option<Vec<String>>,
    pub class_patterns: Option<Vec<String>>,
    /// Opt-in fields from direct declaration -> container -> member children.
    /// Missing/empty rules leave ordinary field extraction unchanged.
    pub class_field_rules: Option<Vec<ClassFieldRule>>,
    pub function_patterns: Option<Vec<String>>,
    /// Anonymous callable node kinds; functions below these nodes are never free functions.
    pub callable_scope_patterns: Option<Vec<String>>,
    /// Member-only owners not represented as classes (e.g. traits or object literals).
    /// Suppress free-function capture in these subtrees when no class owner exists.
    pub member_scope_patterns: Option<Vec<String>>,
    /// Namespace patterns whose names also apply to following siblings (e.g. package headers).
    pub namespace_header_patterns: Option<Vec<String>>,
    pub variable_patterns: Option<Vec<String>>,
    pub identifier_patterns: Option<Vec<String>>,
    pub type_patterns: Option<Vec<String>>,
    /// Nodes whose extracted type gains an array dimension, such as Java varargs.
    pub array_type_patterns: Option<Vec<String>>,
    pub parameter_container_patterns: Option<Vec<String>>,
    pub parameter_patterns: Option<Vec<String>>,
    pub wrapper_patterns: Option<Vec<String>>,
    pub skip_patterns: Option<Vec<String>>,
    pub import_patterns: Option<Vec<String>>,
    pub namespace_patterns: Option<Vec<String>>,
    pub visibility_modifier_patterns: Option<Vec<String>>,
    pub private_by_default: Option<bool>,
    pub public_modifier_patterns: Option<Vec<String>>,
    pub private_modifier_patterns: Option<Vec<String>>,
    pub class_type_parameter_patterns: Option<Vec<String>>,
    /// Field names (per the grammar's own field-name convention, e.g. tree-sitter's
    /// `field_name_for_child`) that mark a child as a type reference rather than a declared
    /// name. Skipped during identifier extraction — needed for grammars (e.g. C#) that reuse
    /// the same node kind (`identifier`) for both a type reference and a declared name.
    pub type_field_names: Option<Vec<String>>,
    /// Node kinds that carry a type's generic arguments as a *sibling* of the type-name node
    /// rather than inside it (e.g. Dart's `List<int>` parses as `type_identifier` +
    /// `type_arguments`). Merged into the type text before generics are split out.
    pub type_argument_patterns: Option<Vec<String>>,
    pub type_path_separator: Option<String>,
    /// Trailing text a grammar leaves attached to a type's own node that isn't part of the type
    /// name — e.g. the pointer star in Objective-C/C's `type_name` for `NSString *`. Stripped
    /// (repeatedly, along with surrounding whitespace) from both a type and its generic
    /// arguments. Absent means nothing is stripped.
    pub type_strip_suffixes: Option<Vec<String>>,
    pub self_parameter_patterns: Option<Vec<String>>,
    pub type_annotation_strip_prefix: Option<String>,
    /// Node kinds that are constructors: suppress their return-type display (no trailing `void`).
    pub constructor_patterns: Option<Vec<String>>,
    /// Language has field-promoting parameters — a parameter that declares no type because it
    /// names an existing field of the enclosing class (e.g. Dart's `Box(this.inner)`). When set,
    /// an untyped parameter adopts the type of the same-named field already collected on the
    /// enclosing class. See `Diagram::infer_param_types_from_fields`.
    pub infer_param_types_from_fields: Option<bool>,
    /// Drop association/dependency edges where the source and destination class are the same.
    pub suppress_self_relations: Option<bool>,
    /// Draw an association edge to a field's type even when that class isn't defined in the
    /// parsed diagram (e.g. an external/imported type). Skipped for generic containers (types
    /// with inner type params), lowercase-leading primitive keywords, and names listed in
    /// `builtin_type_names`.
    pub infer_unresolved_type_relations: Option<bool>,
    /// Well-known standard-library/builtin type names (e.g. `String`, `Object`) that should never
    /// be treated as a user-defined class reference, even though they're capitalized.
    pub builtin_type_names: Option<Vec<String>>,
    /// Language has no real type annotations (e.g. JavaScript). `extract_type` falls back to a
    /// `void` placeholder when it finds nothing, which is correct for statically-typed languages
    /// with an actual void return, but meaningless here — writers suppress it instead of
    /// printing `+id: void` / `add() void`. See `Variable::render` / `Function::render`.
    pub loosely_typed: Option<bool>,
}

/// Extract fields only from matching direct children of a matching class owner.
#[derive(Clone, Deserialize, Debug)]
pub struct ClassFieldRule {
    pub owner_patterns: Vec<String>,
    pub container_patterns: Vec<String>,
    pub member_patterns: Vec<String>,
    /// Defaults to public fields, independently of ordinary visibility extraction.
    #[serde(default)]
    pub private: bool,
    /// Omitted policy generates no getters.
    pub getter: Option<GetterConfig>,
}

/// Only public, same-name accessors are supported by the current function model.
/// Unsupported policy strings fail configuration deserialization.
#[derive(Clone, Deserialize, Debug, Default)]
pub struct GetterConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub visibility: GetterVisibility,
    #[serde(default)]
    pub naming: GetterNaming,
}

#[derive(Clone, Deserialize, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub enum GetterVisibility {
    #[default]
    Public,
}

#[derive(Clone, Deserialize, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub enum GetterNaming {
    #[default]
    SameAsField,
}

impl LangConfig {
    pub fn all_configs() -> Vec<(String, Self)> {
        let mut configs = Vec::new();
        for dir in LANG_DIR.dirs() {
            if let Some(lang_name) = dir.path().file_name().and_then(|n| n.to_str()) {
                let config = Self::load(lang_name);
                configs.push((lang_name.to_string(), config));
            }
        }
        configs
    }

    pub fn load(language: &str) -> Self {
        let lang_dir = match language.to_lowercase().as_str() {
            "rust" => "rust",
            "java" => "java",
            "javascript" | "js" => "javascript",
            "typescript" | "ts" => "typescript",
            "cpp" | "c++" => "cpp",
            "csharp" | "cs" | "c-sharp" => "csharp",
            "objective-c" | "objc" => "objc",
            "dart" => "dart",
            "kotlin" | "kt" => "kotlin",
            _ => language,
        };

        let config_path = Path::new(lang_dir).join("config.yaml");

        if let Some(file) = LANG_DIR.get_file(config_path) {
            if let Some(content) = file.contents_utf8() {
                match serde_yml::from_str::<LangConfig>(content) {
                    Ok(config) => return config,
                    Err(e) => {
                        eprintln!("Error parsing embedded config for {}: {}", language, e);
                    }
                }
            }
        } else {
            eprintln!(
                "Warning: Could not find embedded config for language: {}",
                language
            );
        }

        // Return a default empty config if loading fails
        LangConfig::default()
    }

    pub fn list_languages() -> Vec<&'static str> {
        vec![
            "rust",
            "java",
            "javascript",
            "typescript",
            "cpp",
            "csharp",
            "objective-c",
            "dart",
            "kotlin",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn omitted_getter_policy_is_absent() {
        let rule: ClassFieldRule =
            serde_yml::from_str("owner_patterns: []\ncontainer_patterns: []\nmember_patterns: []")
                .unwrap();
        assert!(rule.getter.is_none());
    }

    #[test]
    fn empty_getter_policy_is_disabled() {
        let getter: GetterConfig = serde_yml::from_str("{}").unwrap();
        assert!(!getter.enabled);
    }

    #[test]
    fn enabled_getter_policy_uses_documented_defaults() {
        let getter: GetterConfig = serde_yml::from_str("enabled: true").unwrap();
        assert!(getter.enabled);
        assert!(matches!(getter.visibility, GetterVisibility::Public));
        assert!(matches!(getter.naming, GetterNaming::SameAsField));
    }

    #[test]
    fn unsupported_getter_policies_are_rejected() {
        for yaml in ["visibility: private", "naming: get_prefix"] {
            assert!(serde_yml::from_str::<GetterConfig>(yaml).is_err(), "{yaml}");
        }
    }
}
