use auto_uml::diagram::Diagram;
use auto_uml::lang_config::LangConfig;
use auto_uml::mermaid::generate;
use auto_uml::stitcher::Stitcher;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tree_sitter::Parser;

/// Configuration for a specific language's integration tests.
#[derive(Deserialize, Debug)]
struct LangTestConfig {
    name: String,
    path_prefix: String,
    /// Mapping of (Test Category) -> (File Name)
    files: HashMap<String, String>,
}

fn load_single_lang_config(lang_dir: &str) -> LangTestConfig {
    let config_path = format!("languages/{}/test.yaml", lang_dir);
    let content = fs::read_to_string(&config_path)
        .unwrap_or_else(|_| panic!("Could not find test config at {}", config_path));
    serde_yml::from_str::<LangTestConfig>(&content)
        .unwrap_or_else(|e| panic!("Error parsing test config for {}: {}", lang_dir, e))
}

fn setup_parser(lang: &str) -> Parser {
    let mut parser = Parser::new();
    match lang {
        "rust" => {
            parser
                .set_language(&tree_sitter_rust::LANGUAGE.into())
                .expect("Error loading Rust grammar");
        }
        "java" => {
            parser
                .set_language(&tree_sitter_java::LANGUAGE.into())
                .expect("Error loading Java grammar");
        }
        "javascript" => {
            parser
                .set_language(&tree_sitter_javascript::LANGUAGE.into())
                .expect("Error loading JavaScript grammar");
        }
        "csharp" => {
            parser
                .set_language(&tree_sitter_c_sharp::LANGUAGE.into())
                .expect("Error loading C# grammar");
        }
        "cpp" => {
            parser
                .set_language(&tree_sitter_cpp::LANGUAGE.into())
                .expect("Error loading C++ grammar");
        }
        "typescript" => {
            parser
                .set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
                .expect("Error loading TypeScript grammar");
        }
        "objective-c" => {
            parser
                .set_language(&tree_sitter_objc::LANGUAGE.into())
                .expect("Error loading Objective-C grammar");
        }
        "dart" => {
            parser
                .set_language(&tree_sitter_dart::LANGUAGE.into())
                .expect("Error loading Dart grammar");
        }
        "kotlin" => {
            parser
                .set_language(&tree_sitter_kotlin::LANGUAGE.into())
                .expect("Error loading Kotlin grammar");
        }
        _ => panic!("Unsupported language: {}", lang),
    }
    parser
}

fn get_file_for_category(config: &LangTestConfig, category: &str) -> String {
    let file = config
        .files
        .get(category)
        .expect("No file defined for category");
    format!("{}/{}", config.path_prefix, file)
}

fn get_uml_for_category(config: &LangTestConfig, category: &str) -> String {
    let source_path = get_file_for_category(config, category);
    source_path
        .rsplit_once('.')
        .map(|(base, _)| format!("{}.uml", base))
        .expect("Source file has no extension")
}

fn run_test(
    config: &LangTestConfig,
    category: &str,
    validator: impl FnOnce(&Diagram, &LangTestConfig),
) {
    let mut parser = setup_parser(&config.name);
    let path = get_file_for_category(config, category);
    let source = std::fs::read(&path).expect("failed to read test file");
    let lang_config = LangConfig::load(&config.name);
    let mut diagram = Diagram::new(&lang_config);
    diagram.build(&source, &mut parser, None);

    let uml_path = get_uml_for_category(config, category);
    let expected_uml = std::fs::read_to_string(&uml_path).expect("failed to read uml file");

    let generated_uml = generate(&diagram);
    let generated_uml = generated_uml.trim();
    let expected_uml = expected_uml.trim();

    if generated_uml != expected_uml {
        eprintln!(
            "\n\x1b[1;31m--- Mermaid Mismatch for {} - {} ---\x1b[0m",
            config.name, category
        );
        eprintln!("\x1b[1;32m+++ Expected:\x1b[0m\n{}", expected_uml);
        eprintln!("\x1b[1;31m--- Generated:\x1b[0m\n{}", generated_uml);
        eprintln!("\x1b[1;34m-------------------------------------------\x1b[0m\n");
    }

    pretty_assertions::assert_eq!(
        generated_uml,
        expected_uml,
        "Mermaid output mismatch for {} - {}",
        config.name,
        category
    );

    validator(&diagram, config);
}

macro_rules! lang_tests {
    ($mod_name:ident, $lang_dir:literal) => {
        mod $mod_name {
            fn config() -> super::LangTestConfig {
                super::load_single_lang_config($lang_dir)
            }

            #[test]
            fn simple_struct() {
                let cfg = config();
                super::run_test(&cfg, "simple_struct", |diagram, c| {
                    let class = diagram
                        .classes
                        .iter()
                        .find(|cl| cl.name == "User")
                        .expect("Class 'User' not found");
                    assert_eq!(
                        class.variables.len(),
                        3,
                        "Variable count mismatch for {}",
                        c.name
                    );
                    let output = super::generate(diagram);
                    assert!(output.contains("classDiagram"));
                    assert!(output.contains("User"));
                });
            }

            #[test]
            fn impl_block() {
                let cfg = config();
                super::run_test(&cfg, "impl_block", |diagram, c| {
                    let class = diagram
                        .classes
                        .iter()
                        .find(|cl| cl.name == "Calculator")
                        .expect("Class 'Calculator' not found");
                    assert!(
                        class.functions.len() >= 2,
                        "Function count low for {}",
                        c.name
                    );
                    assert!(class.functions.iter().any(|f| f.name == "add"));
                });
            }

            #[test]
            fn complex_types() {
                let cfg = config();
                super::run_test(&cfg, "complex_types", |diagram, _c| {
                    let class = diagram
                        .classes
                        .iter()
                        .find(|cl| cl.name == "ComplexData")
                        .expect("Class 'ComplexData' not found");
                    assert!(
                        class
                            .variables
                            .iter()
                            .any(|v| v.name.as_deref() == Some("raw_bytes"))
                    );
                    assert!(class.functions.iter().any(|f| f.name == "process"));
                });
            }

            #[test]
            fn generics() {
                let cfg = config();
                super::run_test(&cfg, "generics", |diagram, _c| {
                    let class = diagram
                        .classes
                        .iter()
                        .find(|cl| cl.name.contains("Box"))
                        .expect("Generic class 'Box' not found");
                    assert!(
                        class
                            .variables
                            .iter()
                            .any(|v| v.name.as_deref() == Some("inner"))
                    );
                });
            }
        }
    };
}

lang_tests!(rust, "rust");
lang_tests!(java, "java");
lang_tests!(javascript, "javascript");
lang_tests!(csharp, "csharp");
lang_tests!(cpp, "cpp");
lang_tests!(typescript, "typescript");
lang_tests!(objc, "objc");
lang_tests!(dart, "dart");
lang_tests!(kotlin, "kotlin");

mod java_records {
    use super::*;

    #[test]
    fn records() {
        run_test(&load_single_lang_config("java"), "records", |diagram, _| {
            let point = diagram.classes.iter().find(|c| c.name == "Point").unwrap();
            let getter = point.functions.iter().find(|f| f.name == "x").unwrap();
            assert!(getter.return_type.name.is_none());
            assert!(!getter.is_constructor);
        });
    }

    #[test]
    fn varargs_record_components_are_arrays() {
        let config = LangConfig::load("java");
        for (component, expected_type, dot_type) in [
            ("String... messages", "String[]", "String[]"),
            ("int[]... messages", "int[][]", "int[][]"),
            (
                "java.util.List<String>... messages",
                "java.util.List~String~[]",
                r"java.util.List\\<String\\>[]",
            ),
        ] {
            let mut diagram = Diagram::new(&config);
            diagram.build(
                format!("record Messages({component}) {{}}").as_bytes(),
                &mut setup_parser("java"),
                None,
            );
            let messages = &diagram.classes[0];
            assert_eq!(
                messages
                    .variables
                    .iter()
                    .map(|field| field.render(&config))
                    .collect::<Vec<_>>(),
                [format!("-messages: {expected_type}")]
            );
            assert_eq!(
                messages
                    .functions
                    .iter()
                    .map(|function| function.render(&config))
                    .collect::<Vec<_>>(),
                [format!("messages() {expected_type}")]
            );
            assert!(generate(&diagram).contains(&format!("messages() {expected_type}")));
            assert!(
                auto_uml::graphviz::generate(&diagram).contains(&format!("messages() {dot_type}"))
            );
        }
    }

    #[test]
    fn optional_class_field_rules() {
        let enabled = LangConfig::load("java");
        let mut disabled = enabled.clone();
        disabled.class_field_rules.as_mut().unwrap()[0]
            .getter
            .as_mut()
            .unwrap()
            .enabled = false;
        let mut absent = enabled.clone();
        absent.class_field_rules = None;

        for (name, config, expected_fields, expected_getters) in [
            ("no rules", absent, 0, 0),
            ("getter disabled", disabled, 1, 0),
            ("getter enabled", enabled, 1, 1),
        ] {
            let mut diagram = Diagram::new(&config);
            diagram.build(b"record Item(int id) {}", &mut setup_parser("java"), None);
            let item = diagram.classes.iter().find(|c| c.name == "Item").unwrap();
            assert_eq!(item.variables.len(), expected_fields, "{name}");
            assert_eq!(item.functions.len(), expected_getters, "{name}");
        }
    }

    #[test]
    fn class_field_rules_match_only_direct_children() {
        let mut config = LangConfig::load("java");
        config.class_field_rules.as_mut().unwrap()[0].owner_patterns =
            vec!["class_declaration".into()];
        let mut diagram = Diagram::new(&config);
        diagram.build(
            b"class Ordinary { void method(int argument) {} }",
            &mut setup_parser("java"),
            None,
        );
        assert!(diagram.classes[0].variables.is_empty());
    }

    #[test]
    fn overlapping_rules_do_not_duplicate_components() {
        let mut config = LangConfig::load("java");
        let rule = config.class_field_rules.as_ref().unwrap()[0].clone();
        config.class_field_rules.as_mut().unwrap().push(rule);
        let mut diagram = Diagram::new(&config);
        diagram.build(b"record Item(int id) {}", &mut setup_parser("java"), None);
        let item = &diagram.classes[0];
        assert_eq!(item.variables.len(), 1);
        assert_eq!(item.functions.len(), 1);
    }

    #[test]
    fn cross_file_types_and_relationships() {
        let config = LangConfig::load("java");
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("languages/java/record_stitch");
        let mut stitcher = Stitcher::new(root, &config, setup_parser("java"), false);
        let mut directory = stitcher.build();
        directory.merge_all();
        directory.resolve_types(&stitcher.type_map);
        let diagram = &directory.merged_diagram;
        let contact = diagram
            .classes
            .iter()
            .find(|c| c.name == "Contact")
            .unwrap();
        let field = |name| {
            contact
                .variables
                .iter()
                .find(|v| v.name.as_deref() == Some(name))
                .unwrap()
        };
        let getter = |name| {
            contact
                .functions
                .iter()
                .find(|f| f.name == name && !f.is_constructor && f.arguments.is_empty())
                .unwrap()
        };
        assert_eq!(field("address").var_type, "models_Address");
        assert_eq!(
            getter("address").return_type.var_type,
            field("address").var_type
        );
        assert_eq!(
            field("alternatives").inner_types.as_deref().unwrap(),
            ["models_Address"]
        );
        assert_eq!(
            getter("alternatives").return_type.inner_types,
            field("alternatives").inner_types
        );
        let mermaid = generate(diagram);
        assert!(mermaid.contains("Contact --> models_Address"));
        assert!(mermaid.contains("Contact ..> models_Address"));
        let dot = auto_uml::graphviz::generate(diagram);
        assert!(dot.contains("\"Contact\" -> \"models_Address\" [arrowhead=\"vee\"];"));
        assert!(
            dot.contains(
                "\"Contact\" -> \"models_Address\" [arrowhead=\"vee\", style=\"dashed\"];"
            )
        );
    }
}

#[test]
fn test_stitcher_integration() {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.push("languages");
    root.push("stitch_test");

    let lang_config = LangConfig::load("rust");
    let mut stitcher = Stitcher::new(root, &lang_config, setup_parser("rust"), false);
    let mut directory = stitcher.build();
    directory.merge_all();
    directory.resolve_types(&stitcher.type_map);

    let diagram = &directory.merged_diagram;

    // Verify all classes were found with qualified names
    // Path-based qualification: models_User, models_Post, auth_User, App
    let class_names: Vec<String> = diagram.classes.iter().map(|c| c.name.clone()).collect();

    assert!(class_names.contains(&"models_User".to_string()));
    assert!(class_names.contains(&"models_Post".to_string()));
    assert!(class_names.contains(&"auth_User".to_string()));
    assert!(class_names.contains(&"App".to_string()));

    // Verify type resolution in 'App'
    let app_class = diagram.classes.iter().find(|c| c.name == "App").unwrap();

    // latest_post: Post -> resolved to models_Post
    let latest_post_var = app_class
        .variables
        .iter()
        .find(|v| v.name.as_deref() == Some("latest_post"))
        .unwrap();
    assert_eq!(latest_post_var.var_type, "models_Post");

    let current_user_var = app_class
        .variables
        .iter()
        .find(|v| v.name.as_deref() == Some("current_user"))
        .unwrap();
    assert!(current_user_var.var_type == "models_User" || current_user_var.var_type == "auth_User");

    // Verify edge generation in Mermaid
    let output = generate(diagram);
    assert!(output.contains("App --> models_Post"));
}
