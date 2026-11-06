//! Repository tasks for nsh-m3e, run with `cargo xtask <task>`.
//!
//! - `gen-tokens`: generate `crates/nsh-m3e/src/tokens/androidx/` from the vendored androidx token files (ADR 0003).
//! - `gen-tokens --check`: fail if the generated files differ from what is checked in. CI runs this.

mod kotlin;
mod rust;

use std::collections::BTreeSet;
use std::fmt;
use std::fs;
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// A vendored token file that is turned into a Rust module.
///
/// The list grows with the components that need the tokens; a file not listed here is not generated.
struct Source {
    /// The file in `reference/androidx-material3-tokens/`. Its object has the same name.
    file: &'static str,
    /// The module's documentation after its title, one paragraph per element.
    doc: &'static [&'static str],
}

const SOURCES: &[Source] = &[
    Source {
        file: "ExpressiveMotionTokens.kt",
        doc: &[
            "The springs of the Expressive motion scheme: a stiffness and a damping *ratio* each, at unit mass. NSH's \
             `Spring` takes a damping *coefficient*: use these through [`crate::motion::SpringToken`], which \
             converts them (ADR 0004).",
        ],
    },
    Source {
        file: "MotionTokens.kt",
        doc: &[
            "Durations, in milliseconds (the unit of NSH's `Timing::duration_ms`), and easing curves.",
        ],
    },
];

const INDEX_DOC: &[&str] = &[
    "The Material 3 tokens, generated from the androidx Kotlin files vendored in \
     `reference/androidx-material3-tokens/` (ADR 0003).",
    "Each constant names its androidx token in its documentation. Numbers are `f32`, the type NSH takes; the \
     generator refuses a value that `f32` cannot hold exactly.",
];

const REFERENCE_DIR: &str = "reference/androidx-material3-tokens";
const OUTPUT_DIR: &str = "crates/nsh-m3e/src/tokens/androidx";

const USAGE: &str = "usage: cargo xtask gen-tokens [--check]";

#[derive(Debug)]
enum Error {
    Usage(String),
    NoWorkspaceRoot,
    Io {
        path: PathBuf,
        source: io::Error,
    },
    Parse {
        path: PathBuf,
        source: kotlin::ParseError,
    },
    WrongObject {
        path: PathBuf,
        found: String,
    },
    Outdated(Vec<PathBuf>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Usage(message) => write!(f, "{message}\n{USAGE}"),
            Error::NoWorkspaceRoot => write!(f, "cannot find the workspace root"),
            Error::Io { path, source } => write!(f, "{}: {source}", path.display()),
            Error::Parse { path, source } => write!(f, "{}: {source}", path.display()),
            Error::WrongObject { path, found } => write!(
                f,
                "{}: the file defines `{found}`, not the object its name says",
                path.display()
            ),
            Error::Outdated(paths) => {
                writeln!(
                    f,
                    "the generated tokens are out of date; run `cargo xtask gen-tokens`:"
                )?;
                for path in paths {
                    writeln!(f, "  {}", path.display())?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Mode {
    Write,
    Check,
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let _ = writeln!(io::stderr(), "xtask: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &[String]) -> Result<(), Error> {
    let mode = mode(args)?;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or(Error::NoWorkspaceRoot)?;
    gen_tokens(root, mode)
}

fn mode(args: &[String]) -> Result<Mode, Error> {
    match args {
        [task] if task == "gen-tokens" => Ok(Mode::Write),
        [task, flag] if task == "gen-tokens" && flag == "--check" => Ok(Mode::Check),
        _ => Err(Error::Usage(format!("unknown arguments {args:?}"))),
    }
}

/// Generates every module in memory, then writes or compares them all, so a failure leaves no half-written state.
fn gen_tokens(root: &Path, mode: Mode) -> Result<(), Error> {
    let output_dir = root.join(OUTPUT_DIR);
    let mut files = Vec::with_capacity(SOURCES.len() + 1);
    let mut modules = Vec::with_capacity(SOURCES.len());
    for source in SOURCES {
        let path = root.join(REFERENCE_DIR).join(source.file);
        let text = read(&path)?;
        let object = kotlin::parse(&text).map_err(|source| Error::Parse {
            path: path.clone(),
            source,
        })?;
        if Some(object.name.as_str()) != source.file.strip_suffix(".kt") {
            return Err(Error::WrongObject {
                path,
                found: object.name,
            });
        }
        let module = rust::snake(&object.name);
        files.push((
            output_dir.join(format!("{module}.rs")),
            rust::module(&object, source.file, source.doc),
        ));
        modules.push(module);
    }
    files.push((output_dir.join("mod.rs"), rust::index(&modules, INDEX_DOC)));

    let expected: BTreeSet<PathBuf> = files.iter().map(|(path, _)| path.clone()).collect();
    let stale = existing_files(&output_dir)?
        .into_iter()
        .filter(|path| !expected.contains(path));
    match mode {
        Mode::Check => {
            let mut outdated: Vec<PathBuf> = stale.collect();
            for (path, content) in &files {
                let current = match fs::read_to_string(path) {
                    Ok(current) => Some(current),
                    Err(error) if error.kind() == io::ErrorKind::NotFound => None,
                    Err(source) => {
                        return Err(Error::Io {
                            path: path.clone(),
                            source,
                        });
                    }
                };
                if current.as_deref() != Some(content.as_str()) {
                    outdated.push(path.clone());
                }
            }
            if outdated.is_empty() {
                Ok(())
            } else {
                outdated.sort();
                Err(Error::Outdated(outdated))
            }
        }
        Mode::Write => {
            for path in stale {
                fs::remove_file(&path).map_err(|source| Error::Io { path, source })?;
            }
            fs::create_dir_all(&output_dir).map_err(|source| Error::Io {
                path: output_dir.clone(),
                source,
            })?;
            for (path, content) in files {
                fs::write(&path, content).map_err(|source| Error::Io { path, source })?;
            }
            Ok(())
        }
    }
}

fn read(path: &Path) -> Result<String, Error> {
    fs::read_to_string(path).map_err(|source| Error::Io {
        path: path.to_owned(),
        source,
    })
}

/// The files already in the output folder; none when it does not exist yet.
fn existing_files(dir: &Path) -> Result<Vec<PathBuf>, Error> {
    let io_error = |source| Error::Io {
        path: dir.to_owned(),
        source,
    };
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => return Err(io_error(source)),
    };
    let mut files = Vec::new();
    for entry in entries {
        files.push(entry.map_err(io_error)?.path());
    }
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|arg| (*arg).to_owned()).collect()
    }

    #[test]
    fn accepts_only_the_known_arguments() {
        for bad in [
            &[][..],
            &["gen"],
            &["gen-tokens", "--force"],
            &["gen-tokens", "--check", "x"],
        ] {
            assert!(matches!(mode(&args(bad)), Err(Error::Usage(_))), "{bad:?}");
        }
        assert!(matches!(run(&args(&["gen"])), Err(Error::Usage(_))));
        assert_eq!(mode(&args(&["gen-tokens"])).unwrap(), Mode::Write);
        assert_eq!(
            mode(&args(&["gen-tokens", "--check"])).unwrap(),
            Mode::Check
        );
    }

    /// A throwaway workspace holding a copy of the listed sources, removed when dropped.
    struct Scratch(PathBuf);

    impl Scratch {
        fn new(name: &str) -> Self {
            let dir =
                std::env::temp_dir().join(format!("nsh-m3e-xtask-{name}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);
            let real_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
            fs::create_dir_all(dir.join(REFERENCE_DIR)).unwrap();
            for source in SOURCES {
                fs::copy(
                    real_root.join(REFERENCE_DIR).join(source.file),
                    dir.join(REFERENCE_DIR).join(source.file),
                )
                .unwrap();
            }
            Scratch(dir)
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn outdated(result: Result<(), Error>) -> Vec<PathBuf> {
        match result {
            Err(Error::Outdated(paths)) => paths,
            other => panic!("expected Outdated, got {other:?}"),
        }
    }

    #[test]
    fn check_passes_after_generating_and_fails_on_any_drift() {
        let scratch = Scratch::new("drift");
        let root = scratch.0.as_path();
        let output = root.join(OUTPUT_DIR);

        // Nothing generated yet: every file is missing.
        assert_eq!(
            outdated(gen_tokens(root, Mode::Check)).len(),
            SOURCES.len() + 1
        );

        gen_tokens(root, Mode::Write).unwrap();
        gen_tokens(root, Mode::Check).unwrap();

        // A hand edit is caught.
        let edited = output.join("motion_tokens.rs");
        let text = fs::read_to_string(&edited).unwrap();
        fs::write(&edited, text.replace("700.0", "750.0")).unwrap();
        assert_eq!(
            outdated(gen_tokens(root, Mode::Check)),
            vec![edited.clone()]
        );

        // So is a stray file, and writing again removes it and restores the edit.
        let stray = output.join("stray.rs");
        fs::write(&stray, "").unwrap();
        assert_eq!(
            outdated(gen_tokens(root, Mode::Check)),
            vec![edited, stray.clone()]
        );
        gen_tokens(root, Mode::Write).unwrap();
        assert!(!stray.exists());
        gen_tokens(root, Mode::Check).unwrap();
    }

    #[test]
    fn a_changed_source_changes_the_output() {
        let scratch = Scratch::new("source");
        let root = scratch.0.as_path();
        gen_tokens(root, Mode::Write).unwrap();
        let source = root.join(REFERENCE_DIR).join("ExpressiveMotionTokens.kt");
        let text = fs::read_to_string(&source).unwrap();
        fs::write(
            &source,
            text.replace(
                "SpringDefaultSpatialDamping = 0.8f",
                "SpringDefaultSpatialDamping = 0.9f",
            ),
        )
        .unwrap();
        assert_eq!(
            outdated(gen_tokens(root, Mode::Check)),
            vec![root.join(OUTPUT_DIR).join("expressive_motion_tokens.rs")]
        );
    }

    #[test]
    fn an_unsupported_source_line_fails_the_generation() {
        let scratch = Scratch::new("unsupported");
        let root = scratch.0.as_path();
        let source = root.join(REFERENCE_DIR).join("MotionTokens.kt");
        let text = fs::read_to_string(&source).unwrap();
        fs::write(
            &source,
            text.replace(
                "const val DurationShort1 = 50.0",
                "val DurationShort1 = 50.dp",
            ),
        )
        .unwrap();
        assert!(matches!(
            gen_tokens(root, Mode::Write),
            Err(Error::Parse { .. })
        ));
        assert!(
            !root.join(OUTPUT_DIR).exists(),
            "nothing is written when a source fails"
        );
    }

    #[test]
    fn an_unreadable_output_is_an_error_not_a_missing_file() {
        let scratch = Scratch::new("unreadable");
        let root = scratch.0.as_path();
        let output = root.join(OUTPUT_DIR);
        gen_tokens(root, Mode::Write).unwrap();

        // A generated file that cannot be read is reported as such, never as merely outdated.
        let module = output.join("motion_tokens.rs");
        fs::remove_file(&module).unwrap();
        fs::create_dir(&module).unwrap();
        match gen_tokens(root, Mode::Check) {
            Err(Error::Io { path, .. }) => assert_eq!(path, module),
            other => panic!(
                "expected an I/O error on {}, got {other:?}",
                module.display()
            ),
        }

        // The same for the output folder itself.
        fs::remove_dir_all(&output).unwrap();
        fs::write(&output, "").unwrap();
        match gen_tokens(root, Mode::Check) {
            Err(Error::Io { path, .. }) => assert_eq!(path, output),
            other => panic!(
                "expected an I/O error on {}, got {other:?}",
                output.display()
            ),
        }
    }

    #[test]
    fn errors_say_what_failed_and_where() {
        let parse = Error::Parse {
            path: PathBuf::from("A.kt"),
            source: kotlin::ParseError {
                line: 7,
                kind: kotlin::ParseErrorKind::Duplicate("B".into()),
            },
        };
        assert_eq!(parse.to_string(), "A.kt: line 7: `B` is defined twice");
        let outdated = Error::Outdated(vec![PathBuf::from("a.rs"), PathBuf::from("b.rs")]);
        assert_eq!(
            outdated.to_string(),
            "the generated tokens are out of date; run `cargo xtask gen-tokens`:\n  a.rs\n  b.rs\n"
        );
        let wrong = Error::WrongObject {
            path: PathBuf::from("A.kt"),
            found: "B".into(),
        };
        assert_eq!(
            wrong.to_string(),
            "A.kt: the file defines `B`, not the object its name says"
        );
        assert_eq!(
            Error::Usage("unknown arguments []".into()).to_string(),
            format!("unknown arguments []\n{USAGE}")
        );
    }

    #[test]
    fn every_listed_source_parses_and_names_its_object() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        for source in SOURCES {
            let text = read(&root.join(REFERENCE_DIR).join(source.file)).unwrap();
            let object = kotlin::parse(&text).unwrap();
            assert_eq!(Some(object.name.as_str()), source.file.strip_suffix(".kt"));
            assert!(!object.tokens.is_empty(), "{}", source.file);
        }
    }
}
