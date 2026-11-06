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
        file: "ButtonLargeTokens.kt",
        doc: &[
            "The large button's size: its container's height and shapes (round, square, pressed and selected), its \
             leading and trailing space, its icon's size and the space between icon and label.",
        ],
    },
    Source {
        file: "ButtonMediumTokens.kt",
        doc: &[
            "The medium button's size: its container's height and shapes (round, square, pressed and selected), its \
             leading and trailing space, its icon's size and the space between icon and label.",
        ],
    },
    Source {
        file: "ButtonSmallTokens.kt",
        doc: &[
            "The small button's size: its container's height and shapes (round, square, pressed and selected), its \
             leading and trailing space, its icon's size and the space between icon and label.",
        ],
    },
    Source {
        file: "ButtonXLargeTokens.kt",
        doc: &[
            "The extra large button's size: its container's height and shapes (round, square, pressed and selected), its \
             leading and trailing space, its icon's size and the space between icon and label.",
        ],
    },
    Source {
        file: "ButtonXSmallTokens.kt",
        doc: &[
            "The extra small button's size: its container's height and shapes (round, square, pressed and selected), its \
             leading and trailing space, its icon's size and the space between icon and label.",
        ],
    },
    Source {
        file: "ColorDarkTokens.kt",
        doc: &[
            "The baseline dark color scheme: each color role as a tone of [`super::palette_tokens`].",
        ],
    },
    Source {
        file: "ColorLightTokens.kt",
        doc: &[
            "The baseline light color scheme: each color role as a tone of [`super::palette_tokens`].",
        ],
    },
    Source {
        file: "ElevatedButtonTokens.kt",
        doc: &[
            "The elevated button's colors and elevation in each state. Colors are roles of the scheme \
             ([`crate::color::ColorRole`]), resolved against the light or dark [`crate::color::ColorScheme`].",
        ],
    },
    Source {
        file: "ElevationTokens.kt",
        doc: &[
            "The elevation levels 0 to 5: how far a surface sits above the one under it. androidx gives them in dp, \
             the density-independent pixel, which is NSH's point.",
        ],
    },
    Source {
        file: "ExpressiveMotionTokens.kt",
        doc: &[
            "The springs of the Expressive motion scheme: a stiffness and a damping *ratio* each, at unit mass. NSH's \
             `Spring` takes a damping *coefficient*: use these through [`crate::motion::SpringToken`], which \
             converts them (ADR 0004).",
        ],
    },
    Source {
        file: "FilledButtonTokens.kt",
        doc: &[
            "The filled button's colors and elevation in each state. Colors are roles of the scheme \
             ([`crate::color::ColorRole`]), resolved against the light or dark [`crate::color::ColorScheme`].",
        ],
    },
    Source {
        file: "MotionTokens.kt",
        doc: &[
            "Durations, in milliseconds (the unit of NSH's `Timing::duration_ms`), and easing curves.",
        ],
    },
    Source {
        file: "OutlinedButtonTokens.kt",
        doc: &[
            "The outlined button's colors and elevation in each state. Colors are roles of the scheme \
             ([`crate::color::ColorRole`]), resolved against the light or dark [`crate::color::ColorScheme`].",
        ],
    },
    Source {
        file: "PaletteTokens.kt",
        doc: &[
            "The baseline tonal palettes: each key color (primary, secondary, tertiary, neutral, neutral variant, \
             error) at a range of tones from 0 (black) to 100 (white), in 8-bit sRGB.",
            "Components never draw with a palette color: they draw with a color role of the scheme, which names one \
             of these tones ([`super::color_light_tokens`], [`super::color_dark_tokens`]).",
        ],
    },
    Source {
        file: "ShapeTokens.kt",
        doc: &[
            "The corner shapes, from square (`CORNER_NONE`) to fully round (`CORNER_FULL`), and the corner sizes \
             alone (`CORNER_VALUE_*`), in dp, which is NSH's point.",
            "Components name a shape through an androidx `ShapeKeyTokens` key, which has the name of one of the \
             shapes here: `ShapeKeyTokens.CornerFull` is [`CORNER_FULL`].",
        ],
    },
    Source {
        file: "StateTokens.kt",
        doc: &[
            "The opacities of the state layer: a layer of the content's color drawn over a component while it is \
             hovered, focused, pressed or dragged.",
        ],
    },
    Source {
        file: "TextButtonTokens.kt",
        doc: &[
            "The text button's colors and elevation in each state. Colors are roles of the scheme \
             ([`crate::color::ColorRole`]), resolved against the light or dark [`crate::color::ColorScheme`].",
        ],
    },
    Source {
        file: "TonalButtonTokens.kt",
        doc: &[
            "The tonal button's colors and elevation in each state. Colors are roles of the scheme \
             ([`crate::color::ColorRole`]), resolved against the light or dark [`crate::color::ColorScheme`].",
        ],
    },
    Source {
        file: "TypeScaleTokens.kt",
        doc: &[
            "The type scale: for each style (display, headline, title, body and label, each large, medium and \
             small, and each also emphasized), its font, weight, size, line height and tracking.",
            "Sizes, line heights and tracking are in sp, the scaled pixel, as NSH points. NSH scales a text's size \
             and line height by the user's text size, as Android scales sp; it does not scale tracking.",
        ],
    },
    Source {
        file: "TypefaceTokens.kt",
        doc: &[
            "The typefaces of the type scale (brand and plain) and its three weights (regular, medium and bold).",
        ],
    },
    Source {
        file: "TypographyTokens.kt",
        doc: &[
            "The type styles: each style of the type scale as one value, with its font, weight, size, line height and \
             tracking from [`super::type_scale_tokens`].",
            "androidx's styles also center each line's text in its line height and trim nothing \
             (`DefaultLineHeightStyle`), and let an app give `Typography` a font family of its own; the library \
             keeps the token's family.",
        ],
    },
];

/// Vendored files of keys (`val Primary = ColorToken(25)`) that component tokens name, read to check those names
/// but not turned into modules: the library has its own type for each kind of key.
const KEY_SOURCES: &[&str] = &["ColorSchemeKeyTokens.kt", "ShapeKeyTokens.kt"];

const INDEX_DOC: &[&str] = &[
    "The Material 3 tokens, generated from the androidx Kotlin files vendored in \
     `reference/androidx-material3-tokens/` (ADR 0003).",
    "Each constant names its androidx token in its documentation. Numbers are `f32`, the type NSH takes; the \
     generator refuses a value that `f32` cannot hold exactly, except a text unit (`0.1.sp`), which androidx \
     itself rounds to the nearest `f32`.",
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
    BadReference {
        path: PathBuf,
        token: String,
        kind: kotlin::Kind,
        reference: kotlin::Reference,
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
            Error::BadReference {
                path,
                token,
                kind,
                reference,
            } => write!(
                f,
                "{}: `{token}` refers to `{}.{}`, which is not a generated {kind}",
                path.display(),
                reference.object,
                reference.token
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

/// Reads and parses one vendored file, whose object must have the file's name.
fn read_object(root: &Path, file: &str) -> Result<(PathBuf, kotlin::TokenObject), Error> {
    let path = root.join(REFERENCE_DIR).join(file);
    let text = read(&path)?;
    let object = kotlin::parse(&text).map_err(|source| Error::Parse {
        path: path.clone(),
        source,
    })?;
    if Some(object.name.as_str()) != file.strip_suffix(".kt") {
        return Err(Error::WrongObject {
            path,
            found: object.name,
        });
    }
    Ok((path, object))
}

/// Generates every module in memory, then writes or compares them all, so a failure leaves no half-written state.
fn gen_tokens(root: &Path, mode: Mode) -> Result<(), Error> {
    let output_dir = root.join(OUTPUT_DIR);
    let mut objects = Vec::with_capacity(SOURCES.len() + KEY_SOURCES.len());
    for source in SOURCES {
        objects.push(read_object(root, source.file)?);
    }
    // The key files come last: references are checked against them, and the zip below, which stops with
    // `SOURCES`, never makes a module of them.
    for file in KEY_SOURCES {
        objects.push(read_object(root, file)?);
    }
    check_references(&objects)?;
    let mut files = Vec::with_capacity(SOURCES.len() + 1);
    let mut modules = Vec::with_capacity(SOURCES.len());
    for (source, (_, object)) in SOURCES.iter().zip(&objects) {
        let module = rust::snake(&object.name);
        files.push((
            output_dir.join(format!("{module}.rs")),
            rust::module(object, source.file, source.doc),
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

/// Fails unless every reference names a token of its own kind (a color for a color) in a generated object, so a
/// reference is never emitted to a constant that does not exist or has another type.
fn check_references(objects: &[(PathBuf, kotlin::TokenObject)]) -> Result<(), Error> {
    let find = |reference: &kotlin::Reference| {
        objects
            .iter()
            .find(|(_, other)| other.name == reference.object)
            .and_then(|(_, other)| {
                other
                    .tokens
                    .iter()
                    .find(|candidate| candidate.name == reference.token)
            })
    };
    for (path, object) in objects {
        for token in &object.tokens {
            let bad = |kind: kotlin::Kind, reference: &kotlin::Reference| Error::BadReference {
                path: path.clone(),
                token: token.name.clone(),
                kind,
                reference: reference.clone(),
            };
            // Each property of a text style names a token of its own kind.
            if let kotlin::Value::TextStyle(style) = &token.value {
                for (kind, reference) in style.properties() {
                    if !find(reference).is_some_and(|target| kind.holds(&target.value)) {
                        return Err(bad(kind, reference));
                    }
                }
                continue;
            }
            let kotlin::Value::Reference(kind, reference) = &token.value else {
                continue;
            };
            let target = find(reference);
            // A shape key names the shape of the same name in `ShapeTokens`, which must exist too.
            let shape = (*kind == kotlin::Kind::ShapeKey).then(|| {
                objects
                    .iter()
                    .find(|(_, other)| other.name == "ShapeTokens")
                    .and_then(|(_, other)| {
                        other
                            .tokens
                            .iter()
                            .find(|candidate| candidate.name == reference.token)
                    })
            });
            let shape_found = shape.is_none_or(|shape| {
                shape.is_some_and(|shape| matches!(shape.value, kotlin::Value::Shape(_)))
            });
            if !shape_found || !target.is_some_and(|target| kind.holds(&target.value)) {
                return Err(bad(*kind, reference));
            }
        }
    }
    Ok(())
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
            let files = SOURCES
                .iter()
                .map(|source| source.file)
                .chain(KEY_SOURCES.iter().copied());
            for file in files {
                fs::copy(
                    real_root.join(REFERENCE_DIR).join(file),
                    dir.join(REFERENCE_DIR).join(file),
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

    fn object(text: &str) -> (PathBuf, kotlin::TokenObject) {
        let object = kotlin::parse(text).unwrap();
        (PathBuf::from(format!("{}.kt", object.name)), object)
    }

    fn bad_reference(objects: &[(PathBuf, kotlin::TokenObject)]) -> (PathBuf, String, String) {
        match check_references(objects) {
            Err(Error::BadReference {
                path,
                token,
                reference,
                ..
            }) => (
                path,
                token,
                format!("{}.{}", reference.object, reference.token),
            ),
            other => panic!("expected BadReference, got {other:?}"),
        }
    }

    const PALETTE: &str = "internal object PaletteTokens {\n    inline val Black: Color\n        get() = Color(red = 0, green = 0, blue = 0)\n    const val Opacity = 0.5f\n}\n";

    #[test]
    fn a_reference_must_name_a_color_of_a_generated_object() {
        let scheme = |target: &str| {
            object(&format!(
                "internal object SchemeTokens {{\n    inline val A: Color\n        get() = PaletteTokens.Black\n    inline val B: Color\n        get() = {target}\n}}\n"
            ))
        };
        // A color, and a color that is itself a reference, are fine.
        check_references(&[object(PALETTE), scheme("A")]).unwrap();
        check_references(&[object(PALETTE), scheme("PaletteTokens.Black")]).unwrap();

        // A token that does not exist, a number, an object that is not generated.
        assert_eq!(
            bad_reference(&[object(PALETTE), scheme("PaletteTokens.White")]),
            (
                "SchemeTokens.kt".into(),
                "B".into(),
                "PaletteTokens.White".into()
            )
        );
        assert_eq!(
            bad_reference(&[object(PALETTE), scheme("PaletteTokens.Opacity")]),
            (
                "SchemeTokens.kt".into(),
                "B".into(),
                "PaletteTokens.Opacity".into()
            )
        );
        assert_eq!(
            bad_reference(&[scheme("A")]),
            (
                "SchemeTokens.kt".into(),
                "A".into(),
                "PaletteTokens.Black".into()
            )
        );
    }

    #[test]
    fn a_reference_must_name_a_token_of_its_own_kind() {
        let typeface = || {
            object(
                "internal object TypefaceTokens {\n    inline val Plain: FontFamily\n        get() = FontFamily.SansSerif\n    inline val WeightBold: FontWeight\n        get() = FontWeight.Bold\n}\n",
            )
        };
        let scale = |ty: &str, target: &str| {
            object(&format!(
                "internal object ScaleTokens {{\n    inline val X: {ty}\n        get() = TypefaceTokens.{target}\n    inline val Y: {ty}\n        get() = X\n}}\n"
            ))
        };
        // Each kind names its own, directly or through another reference.
        check_references(&[typeface(), scale("FontFamily", "Plain")]).unwrap();
        check_references(&[typeface(), scale("FontWeight", "WeightBold")]).unwrap();
        // A weight that names a family, or a family that names a weight, is refused with its kind.
        for (ty, target, kind) in [
            ("FontWeight", "Plain", kotlin::Kind::FontWeight),
            ("FontFamily", "WeightBold", kotlin::Kind::FontFamily),
        ] {
            match check_references(&[typeface(), scale(ty, target)]) {
                Err(Error::BadReference {
                    token, kind: found, ..
                }) => assert_eq!((token.as_str(), found), ("X", kind)),
                other => panic!("expected BadReference, got {other:?}"),
            }
        }
        let error = Error::BadReference {
            path: PathBuf::from("A.kt"),
            token: "B".into(),
            kind: kotlin::Kind::FontWeight,
            reference: kotlin::Reference {
                object: "C".into(),
                token: "D".into(),
            },
        };
        assert_eq!(
            error.to_string(),
            "A.kt: `B` refers to `C.D`, which is not a generated font weight"
        );
    }

    const COLOR_KEYS: &str = "@JvmInline internal value class ColorToken(val id: Int)\ninternal object ColorSchemeKeyTokens {\n    val Primary = ColorToken(25)\n}\n";
    const SHAPE_KEYS: &str = "@JvmInline internal value class ShapeToken(val id: Int)\ninternal object ShapeKeyTokens {\n    val CornerFull = ShapeToken(5)\n    val CornerSmall = ShapeToken(12)\n}\n";

    fn button(ty: &str, target: &str) -> (PathBuf, kotlin::TokenObject) {
        object(&format!(
            "internal object ButtonTokens {{\n    inline val X: {ty}\n        get() = {target}\n}}\n"
        ))
    }

    fn bad_kind(objects: &[(PathBuf, kotlin::TokenObject)]) -> kotlin::Kind {
        match check_references(objects) {
            Err(Error::BadReference { token, kind, .. }) => {
                assert_eq!(token, "X");
                kind
            }
            other => panic!("expected BadReference, got {other:?}"),
        }
    }

    #[test]
    fn a_color_role_must_name_a_key_of_the_color_scheme() {
        let role = |target| button("ColorToken", target);
        check_references(&[object(COLOR_KEYS), role("ColorSchemeKeyTokens.Primary")]).unwrap();
        // A key the file does not have, or no file of keys at all.
        assert_eq!(
            bad_kind(&[object(COLOR_KEYS), role("ColorSchemeKeyTokens.Tertiary")]),
            kotlin::Kind::ColorRole
        );
        assert_eq!(
            bad_kind(&[role("ColorSchemeKeyTokens.Primary")]),
            kotlin::Kind::ColorRole
        );
        // A key of another class under the same object name is not a color role.
        let shape_keys_named_as_colors =
            object(&SHAPE_KEYS.replace("ShapeKeyTokens", "ColorSchemeKeyTokens"));
        assert_eq!(
            bad_kind(&[
                shape_keys_named_as_colors,
                role("ColorSchemeKeyTokens.CornerFull")
            ]),
            kotlin::Kind::ColorRole
        );
    }

    #[test]
    fn a_shape_key_must_name_a_key_and_a_shape_of_the_same_name() {
        let shapes = |body: &str| object(&format!("internal object ShapeTokens {{\n{body}}}\n"));
        let key = |target| button("ShapeToken", target);
        let full = "    val CornerFull = CircleShape\n";
        check_references(&[
            shapes(full),
            object(SHAPE_KEYS),
            key("ShapeKeyTokens.CornerFull"),
        ])
        .unwrap();
        // A shape with no key, a key with no shape, a key whose namesake is a corner size, not a shape.
        assert_eq!(
            bad_kind(&[
                shapes(full),
                object(SHAPE_KEYS),
                key("ShapeKeyTokens.CornerLarge")
            ]),
            kotlin::Kind::ShapeKey
        );
        assert_eq!(
            bad_kind(&[
                shapes(full),
                object(SHAPE_KEYS),
                key("ShapeKeyTokens.CornerSmall")
            ]),
            kotlin::Kind::ShapeKey
        );
        assert_eq!(
            bad_kind(&[
                shapes("    val CornerFull = CornerSize(4.0.dp)\n"),
                object(SHAPE_KEYS),
                key("ShapeKeyTokens.CornerFull"),
            ]),
            kotlin::Kind::ShapeKey
        );
        assert_eq!(
            bad_kind(&[object(SHAPE_KEYS), key("ShapeKeyTokens.CornerFull")]),
            kotlin::Kind::ShapeKey
        );
    }

    #[test]
    fn a_length_must_name_a_length() {
        let elevation = || {
            object(
                "internal object ElevationTokens {\n    inline val Level1: androidx.compose.ui.unit.Dp\n        get() = 1.0.dp\n    const val Opacity = 1.0\n}\n",
            )
        };
        let length = |target| button("androidx.compose.ui.unit.Dp", target);
        check_references(&[elevation(), length("ElevationTokens.Level1")]).unwrap();
        assert_eq!(
            bad_kind(&[elevation(), length("ElevationTokens.Opacity")]),
            kotlin::Kind::Length
        );
    }

    /// `TypeScaleTokens` with the five tokens of `BodyLarge`, the size given as `size`.
    fn type_scale(size: kotlin::Value) -> (PathBuf, kotlin::TokenObject) {
        let token = |name: &str, value| kotlin::Token {
            name: format!("BodyLarge{name}"),
            value,
        };
        let object = kotlin::TokenObject {
            name: "TypeScaleTokens".into(),
            version: None,
            tokens: vec![
                token(
                    "Font",
                    kotlin::Value::FontFamily(kotlin::FontFamily::SansSerif),
                ),
                token("Weight", kotlin::Value::FontWeight(400)),
                token("Size", size),
                token("LineHeight", kotlin::Value::Sp(24.0)),
                token("Tracking", kotlin::Value::Sp(0.5)),
            ],
        };
        (PathBuf::from("TypeScaleTokens.kt"), object)
    }

    /// `TypographyTokens` with one style, `X`, whose properties name `BodyLarge` + `suffixes` in `TypeScaleTokens`.
    fn typography(suffixes: [&str; 5]) -> (PathBuf, kotlin::TokenObject) {
        let [font, weight, size, line_height, tracking] =
            suffixes.map(|suffix| kotlin::Reference {
                object: "TypeScaleTokens".into(),
                token: format!("BodyLarge{suffix}"),
            });
        let style = kotlin::TextStyle {
            font,
            weight,
            size,
            line_height,
            tracking,
        };
        let object = kotlin::TokenObject {
            name: "TypographyTokens".into(),
            version: None,
            tokens: vec![kotlin::Token {
                name: "X".into(),
                value: kotlin::Value::TextStyle(Box::new(style)),
            }],
        };
        (PathBuf::from("TypographyTokens.kt"), object)
    }

    const STYLE: [&str; 5] = ["Font", "Weight", "Size", "LineHeight", "Tracking"];

    #[test]
    fn each_property_of_a_text_style_must_name_a_token_of_its_kind() {
        let scale = || type_scale(kotlin::Value::Sp(16.0));
        check_references(&[scale(), typography(STYLE)]).unwrap();
        // A token that does not exist, or no type scale at all, is refused for the property that names it.
        assert_eq!(
            bad_kind(&[
                scale(),
                typography(["Font", "Weight", "Size", "Leading", "Tracking"])
            ]),
            kotlin::Kind::TextUnit
        );
        assert_eq!(bad_kind(&[typography(STYLE)]), kotlin::Kind::FontFamily);
        // A family that names a weight, a weight that names a size, a size that names a family.
        assert_eq!(
            bad_kind(&[
                scale(),
                typography(["Weight", "Weight", "Size", "LineHeight", "Tracking"])
            ]),
            kotlin::Kind::FontFamily
        );
        assert_eq!(
            bad_kind(&[
                scale(),
                typography(["Font", "Size", "Size", "LineHeight", "Tracking"])
            ]),
            kotlin::Kind::FontWeight
        );
        assert_eq!(
            bad_kind(&[
                scale(),
                typography(["Font", "Weight", "Font", "LineHeight", "Tracking"])
            ]),
            kotlin::Kind::TextUnit
        );
        // A size in `dp` is a length, not a text unit.
        assert_eq!(
            bad_kind(&[type_scale(kotlin::Value::Dp(16.0)), typography(STYLE)]),
            kotlin::Kind::TextUnit
        );
    }

    #[test]
    fn the_vendored_typography_gives_every_style_of_the_type_scale() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let text = read(&root.join(REFERENCE_DIR).join("TypographyTokens.kt")).unwrap();
        let object = kotlin::parse(&text).unwrap();
        let names: Vec<&str> = object
            .tokens
            .iter()
            .map(|token| token.name.as_str())
            .collect();
        // Display, headline, title, body and label, each large, medium and small, and each emphasized.
        assert_eq!(names.len(), 30, "{names:?}");
        for role in ["Display", "Headline", "Title", "Body", "Label"] {
            for size in ["Large", "Medium", "Small"] {
                for emphasis in ["", "Emphasized"] {
                    let name = format!("{role}{size}{emphasis}");
                    assert!(names.contains(&name.as_str()), "{name} missing: {names:?}");
                }
            }
        }
        assert!(
            object
                .tokens
                .iter()
                .all(|token| matches!(token.value, kotlin::Value::TextStyle(_)))
        );
    }

    #[test]
    fn a_file_of_keys_is_checked_but_never_generated() {
        let scratch = Scratch::new("keys");
        let root = scratch.0.as_path();
        let keys = root.join(REFERENCE_DIR).join("ShapeKeyTokens.kt");
        let text = fs::read_to_string(&keys).unwrap();

        // A key a button names is gone: the generation fails before writing anything.
        fs::write(&keys, text.replace("val CornerFull = ", "val CornerFul = ")).unwrap();
        assert!(matches!(
            gen_tokens(root, Mode::Write),
            Err(Error::BadReference {
                kind: kotlin::Kind::ShapeKey,
                ..
            })
        ));
        assert!(!root.join(OUTPUT_DIR).exists());

        // A file of keys that defines another object is refused like any source.
        fs::write(
            &keys,
            text.replace("object ShapeKeyTokens", "object OtherKeyTokens"),
        )
        .unwrap();
        assert!(matches!(
            gen_tokens(root, Mode::Write),
            Err(Error::WrongObject { found, .. }) if found == "OtherKeyTokens"
        ));

        fs::write(&keys, &text).unwrap();
        gen_tokens(root, Mode::Write).unwrap();
        for file in KEY_SOURCES {
            let module = format!("{}.rs", rust::snake(file.strip_suffix(".kt").unwrap()));
            assert!(!root.join(OUTPUT_DIR).join(module).exists(), "{file}");
        }
    }

    #[test]
    fn a_bad_reference_fails_the_generation_before_anything_is_written() {
        let scratch = Scratch::new("reference");
        let root = scratch.0.as_path();
        let source = root.join(REFERENCE_DIR).join("ColorLightTokens.kt");
        let text = fs::read_to_string(&source).unwrap();
        fs::write(
            &source,
            text.replace("PaletteTokens.Neutral98", "PaletteTokens.Neutral97"),
        )
        .unwrap();
        assert!(matches!(
            gen_tokens(root, Mode::Write),
            Err(Error::BadReference { .. })
        ));
        assert!(!root.join(OUTPUT_DIR).exists());
    }

    #[test]
    fn errors_say_what_failed_and_where() {
        let reference = Error::BadReference {
            path: PathBuf::from("A.kt"),
            token: "B".into(),
            kind: kotlin::Kind::Color,
            reference: kotlin::Reference {
                object: "C".into(),
                token: "D".into(),
            },
        };
        assert_eq!(
            reference.to_string(),
            "A.kt: `B` refers to `C.D`, which is not a generated color"
        );
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
        for file in KEY_SOURCES {
            let text = read(&root.join(REFERENCE_DIR).join(file)).unwrap();
            let object = kotlin::parse(&text).unwrap();
            assert_eq!(Some(object.name.as_str()), file.strip_suffix(".kt"));
            assert!(!object.tokens.is_empty(), "{file}");
            assert!(
                object
                    .tokens
                    .iter()
                    .all(|token| matches!(token.value, kotlin::Value::Key(_))),
                "{file}"
            );
        }
    }
}
