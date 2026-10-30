use auto_uml::input_selection::InputSelection;
use auto_uml::lang_config::LangConfig;
use auto_uml::stitcher::{Directory, Stitcher};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target/multi-input-tests")
            .join(format!(
                "{}-{}",
                std::process::id(),
                NEXT_ID.fetch_add(1, Ordering::Relaxed)
            ));
        fs::create_dir_all(&root).unwrap();
        Self(root)
    }

    fn write(&self, name: &str, content: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, content).unwrap();
        path
    }

    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_auto-uml"));
        command
            .current_dir(&self.0)
            .arg("--destination")
            .arg(self.0.join("output"));
        command
    }

    fn run_args(&self, args: &[&str]) -> Output {
        self.command().args(args).output().unwrap()
    }

    fn run(&self, paths: &[PathBuf], options: &[&str]) -> Output {
        self.command()
            .arg("--source-code")
            .args(paths)
            .args(options)
            .output()
            .unwrap()
    }

    fn output(&self) -> String {
        fs::read_to_string(self.0.join("output")).unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn success(output: Output) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn rust_stitcher(config: &LangConfig) -> Stitcher<'_> {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_rust::LANGUAGE.into())
        .unwrap();
    Stitcher::new(PathBuf::new(), config, parser, false)
}

fn resolved_selection<'a>(selection: &InputSelection, config: &'a LangConfig) -> Directory<'a> {
    let mut stitcher = rust_stitcher(config);
    let mut directory = stitcher.build_selected(selection).unwrap();
    directory.merge_all();
    directory.resolve_types(&stitcher.type_map);
    directory
}

#[test]
fn cli_source_lists_repeated_flags_and_default_directory() {
    let fixture = Fixture::new();
    fixture.write("a.rs", "struct Alpha {}");
    fixture.write("space folder/b.rs", "struct Beta {}");
    for (name, args) in [
        ("list", vec!["-s", "a.rs", "space folder/b.rs"]),
        (
            "repeated flags",
            vec!["-s", "a.rs", "-s", "space folder/b.rs"],
        ),
        ("default directory", vec![]),
    ] {
        let result = fixture.run_args(&args);
        assert!(
            result.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        let output = fixture.output();
        assert!(
            output.contains("Alpha") && output.contains("Beta"),
            "{name}: {output}"
        );
    }
    // Git alone must be accepted without a conflicting default local argument.
    success(fixture.run_args(&["--git", "unused", "--list-languages"]));
}

#[test]
fn cli_rejects_missing_values_and_git_conflicts_before_execution() {
    let fixture = Fixture::new();
    for (name, args, message) in [
        ("missing value", vec!["-s"], "a value is required"),
        (
            "git after list",
            vec!["-s", "a", "b", "--git", "unused"],
            "cannot be used with",
        ),
        (
            "git before source",
            vec!["--git", "unused", "-s", "a"],
            "cannot be used with",
        ),
    ] {
        let result = fixture.run_args(&args);
        assert_eq!(result.status.code(), Some(2), "{name}");
        assert!(
            String::from_utf8_lossy(&result.stderr).contains(message),
            "{name}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(
            !String::from_utf8_lossy(&result.stdout).contains("Cloning"),
            "{name}"
        );
        assert!(!fixture.0.join("output").exists(), "{name}");
    }
}

#[test]
fn cli_renders_selected_cross_root_relationships() {
    let fixture = Fixture::new();
    let account = fixture.write("models/account.rs", "struct Account { user: User }");
    let user = fixture.write("service/user.rs", "struct User {}");
    fixture.write("models/unselected.rs", "struct Unselected {}");
    let paths = [account, user];
    for (format, edge) in [
        ("mermaid", "models_Account --> service_User"),
        ("graphviz", "\"models_Account\" -> \"service_User\""),
    ] {
        success(fixture.run(&paths, &["--format", format]));
        let output = fixture.output();
        assert!(output.contains(edge), "{format}: {output}");
        assert!(!output.contains("Unselected"), "{format}: {output}");
    }
}

#[test]
fn directory_overlap_and_explicit_ignored_files_are_deduplicated() {
    let fixture = Fixture::new();
    fixture.write(".gitignore", "ignored/\n");
    let kept = fixture.write("src/kept.rs", "struct Kept {}");
    let ignored = fixture.write("ignored/file.rs", "struct Ignored {}");
    fixture.write(".hidden.rs", "struct Hidden {}");
    fixture.write("ignored/other.java", "class Java {}");
    let paths = [fixture.0.clone(), fixture.0.join("src"), kept.clone()];
    let selection = InputSelection::collect(&paths, false).unwrap();
    assert_eq!(
        selection
            .files
            .iter()
            .filter(|f| f.path.ends_with("kept.rs"))
            .count(),
        1
    );
    assert!(!selection.files.iter().any(|f| f.path == ignored));
    assert_eq!(selection.detect_language().unwrap(), "rust");
    let selection = InputSelection::collect(&[fixture.0.clone(), ignored.clone()], false).unwrap();
    assert!(
        selection
            .files
            .iter()
            .any(|f| f.path == ignored && f.explicit)
    );
    let selection = InputSelection::collect(&[fixture.0.clone(), kept], true).unwrap();
    assert!(selection.files.iter().any(|f| f.path == ignored));
    assert!(
        selection
            .files
            .iter()
            .any(|f| f.path.ends_with(".hidden.rs"))
    );
}

#[test]
fn folders_mixed_inputs_and_spaces_work() {
    let fixture = Fixture::new();
    let a = fixture.write("space folder/a.rs", "struct Alpha {}");
    fixture.write("other/b.rs", "struct Beta {}");
    let folder = fixture.0.join("space folder");
    let other = fixture.0.join("other");
    success(fixture.run(&[folder, other.clone()], &[]));
    let expected = fixture.output();
    assert!(expected.contains("space_x20_folder_Alpha"));
    success(fixture.run(&[a, other], &[]));
    assert_eq!(fixture.output(), expected);
}

#[test]
fn inference_rejects_mixed_and_ambiguous_languages() {
    let fixture = Fixture::new();
    let rust = fixture.write("a.rs", "struct Rust {}");
    let java = fixture.write("b.java", "class Java {}");
    let header = fixture.write("c.h", "class Header {};");
    for paths in [[rust.clone(), java], [header.clone(), header]] {
        let result = fixture.run(&paths, &[]);
        assert!(!result.status.success());
        assert!(String::from_utf8_lossy(&result.stderr).contains("--lang"));
        assert!(!fixture.0.join("output").exists());
    }
    success(fixture.run(&[fixture.0.clone(), rust], &["--lang", "rust"]));
    assert!(!fixture.output().contains("Java"));
    assert!(!fixture.output().contains("Header"));
}

#[test]
fn explicit_files_use_selected_parser_even_with_other_extensions() {
    let fixture = Fixture::new();
    let a = fixture.write("a.txt", "struct Alpha {}");
    let b = fixture.write("b.txt", "struct Beta {}");
    success(fixture.run(&[a, b], &["--lang", "rust"]));
    assert!(fixture.output().contains("Alpha"));
    assert!(fixture.output().contains("Beta"));
}

#[test]
fn invalid_and_empty_selections_do_not_overwrite_output() {
    let fixture = Fixture::new();
    let valid = fixture.write("a.rs", "struct Alpha {}");
    let empty = fixture.0.join("empty");
    fs::create_dir(&empty).unwrap();
    fixture.write("output", "preserve me");
    for paths in [[valid, fixture.0.join("missing")], [empty.clone(), empty]] {
        assert!(!fixture.run(&paths, &["--lang", "rust"]).status.success());
        assert_eq!(fixture.output(), "preserve me");
    }
}

#[test]
fn selected_read_failure_is_reported() {
    let fixture = Fixture::new();
    let path = fixture.write("a.rs", "struct Alpha {}");
    let selection = InputSelection::collect(&[path.clone(), path.clone()], false).unwrap();
    fs::remove_file(path).unwrap();
    let config = LangConfig::load("rust");
    let mut stitcher = rust_stitcher(&config);
    assert!(stitcher.build_selected(&selection).is_err());
}

#[test]
fn repeated_single_input_preserves_labels_in_both_formats() {
    let fixture = Fixture::new();
    let file = fixture.write("model.rs", "struct Model { value: i32 }");
    for path in [file, fixture.0.clone()] {
        for format in ["mermaid", "graphviz"] {
            success(fixture.run(
                std::slice::from_ref(&path),
                &["--lang", "rust", "--format", format],
            ));
            let expected = fixture.output();
            success(fixture.run(
                &[path.clone(), path.clone()],
                &["--lang", "rust", "--format", format],
            ));
            assert_eq!(fixture.output(), expected);
        }
    }
}

#[test]
fn canonical_identity_handles_dot_dot_aliases() {
    let fixture = Fixture::new();
    let file = fixture.write("src/model.rs", "struct Model {}");
    let alias = fixture.0.join(Path::new("src/../src/model.rs"));
    let selection = InputSelection::collect(&[file, alias], false).unwrap();
    assert_eq!(selection.files.len(), 1);
}

#[test]
fn cli_selection_order_is_stable() {
    let fixture = Fixture::new();
    let a = fixture.write("a.rs", "struct Alpha {}");
    let b = fixture.write("b.rs", "struct Beta {}");
    success(fixture.run(&[b.clone(), a.clone()], &[]));
    let expected = fixture.output();
    success(fixture.run(&[a, b], &[]));
    assert_eq!(fixture.output(), expected);
}

#[cfg(windows)]
#[test]
fn absent_common_base_keeps_drive_qualification_in_valid_identifiers() {
    let fixture = Fixture::new();
    let file = fixture.write("a.rs", "struct Alpha {}");
    let mut selection = InputSelection::collect(&[file], false).unwrap();
    // Exercise cross-drive naming without requiring two writable drives.
    selection.naming_base = None;
    let config = LangConfig::load("rust");
    let directory = resolved_selection(&selection, &config);
    let name = &directory.merged_diagram.classes[0].name;
    let drive = fixture.0.to_string_lossy().chars().next().unwrap();
    assert!(name.starts_with(&format!("{drive}_x3a_")), "{name}");
    assert!(!name.contains(':'));
    assert!(
        directory.merged_diagram.files[0]
            .path
            .as_ref()
            .unwrap()
            .is_absolute()
    );
}

// Include literal escape text and nested paths to guard the identifier encoding
// itself, alongside the two reviewed punctuation and resolution regressions.
const PREFIX_CASES: &[(&str, &str)] = &[
    ("a one", "a_x20_one"),
    ("a two", "a_x20_two"),
    ("a-b", "a_x2d_b"),
    ("a_b", "a_x5f_b"),
    ("a_x2d_b", "a_x5f_x2d_x5f_b"),
    ("a/x2d/b", "a_x2f_x2d_x2f_b"),
    ("_a", "_x5f_a"),
    ("_b", "_x5f_b"),
    #[cfg(unix)]
    (r"a\b", "a_x5c_b"),
    #[cfg(unix)]
    ("a/b", "a_x2f_b"),
];

fn prefix_fixture(cases: &[(&str, &str)]) -> (Fixture, Vec<PathBuf>) {
    let fixture = Fixture::new();
    let paths = cases
        .iter()
        .map(|(folder, _)| {
            fixture.write(
                &format!("{folder}/model.rs"),
                r"
            struct User {}
            struct Account { user: User }
            fn make(user: User) -> User { user }
            fn many(users: Vec<User>) -> Vec<User> { users }
        ",
            )
        })
        .collect();
    (fixture, paths)
}

#[test]
fn selected_prefixes_keep_classes_and_function_contexts_distinct() {
    let (_fixture, paths) = prefix_fixture(PREFIX_CASES);
    let selection = InputSelection::collect(&paths, false).unwrap();
    let config = LangConfig::load("rust");
    let directory = resolved_selection(&selection, &config);
    let diagram = &directory.merged_diagram;
    let names: std::collections::HashSet<_> = diagram.classes.iter().map(|c| &c.name).collect();
    assert_eq!(names.len(), PREFIX_CASES.len() * 2);
    for (folder, prefix) in PREFIX_CASES {
        let user = format!("{prefix}_User");
        let account = diagram
            .classes
            .iter()
            .find(|c| c.name == format!("{prefix}_Account"))
            .unwrap();
        assert_eq!(account.variables[0].var_type, user, "{folder}: field");
        let file = diagram
            .files
            .iter()
            .find(|f| f.path.as_deref() == Some(Path::new(folder).join("model.rs").as_path()))
            .unwrap();
        for free in &file.functions {
            let function = &free.function;
            let expected = if function.name == "many" {
                format!("Vec~{user}~")
            } else {
                user.clone()
            };
            assert_eq!(
                function.return_type.display_type(),
                expected,
                "{folder}: return"
            );
            assert_eq!(
                function.arguments[0].display_type(),
                expected,
                "{folder}: argument"
            );
        }
        assert_eq!(file.functions.len(), 2, "{folder}");
    }
}

#[test]
fn cli_renderers_keep_punctuation_classes_and_relationships_distinct() {
    let cases = [("a-b", "a_x2d_b"), ("a_b", "a_x5f_b")];
    let (fixture, paths) = prefix_fixture(&cases);
    for format in ["mermaid", "graphviz"] {
        success(fixture.run(&paths, &["--format", format]));
        let output = fixture.output();
        for (folder, prefix) in cases {
            let user = format!("{prefix}_User");
            let account = format!("{prefix}_Account");
            let (declaration, edge) = if format == "mermaid" {
                (format!("class {user} {{"), format!("{account} --> {user}"))
            } else {
                (
                    format!("\"{user}\" [label="),
                    format!("\"{account}\" -> \"{user}\""),
                )
            };
            assert!(
                output.contains(&declaration),
                "{format}: {folder}: {output}"
            );
            assert!(output.contains(&edge), "{format}: {folder}: {output}");
        }
    }
}
