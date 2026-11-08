//! Repository tasks for nsh-m3e, run with `cargo xtask <task>`.
//!
//! - `gen-tokens`: generate `crates/nsh-m3e/src/tokens/androidx/` from the vendored androidx token files (ADR 0003).
//! - `gen-tokens --check`: fail if the generated files differ from what is checked in. CI runs this.
//! - `gen-icons`: generate `crates/nsh-m3e/src/material_symbols/` from the vendored Material Symbols (ADR 0017).
//! - `gen-icons --check`: fail if the generated icons differ from what is checked in. CI runs this.

mod icons;
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
    /// The file in `reference/androidx-material3-tokens/`.
    file: &'static str,
    /// The object the file defines, where it is not named after the file: androidx's `IconButtonTokens.kt` defines
    /// `StandardIconButtonTokens`. `None` is the file's name. The module is named after the object.
    object: Option<&'static str>,
    /// The module's documentation after its title, one paragraph per element.
    doc: &'static [&'static str],
}

const SOURCES: &[Source] = &[
    Source {
        file: "ButtonLargeTokens.kt",
        object: None,
        doc: &[
            "The large button's size: its container's height and shapes (round, square, pressed and selected), its \
             leading and trailing space, its icon's size and the space between icon and label.",
        ],
    },
    Source {
        file: "ButtonMediumTokens.kt",
        object: None,
        doc: &[
            "The medium button's size: its container's height and shapes (round, square, pressed and selected), its \
             leading and trailing space, its icon's size and the space between icon and label.",
        ],
    },
    Source {
        file: "ButtonSmallTokens.kt",
        object: None,
        doc: &[
            "The small button's size: its container's height and shapes (round, square, pressed and selected), its \
             leading and trailing space, its icon's size and the space between icon and label.",
        ],
    },
    Source {
        file: "ButtonXLargeTokens.kt",
        object: None,
        doc: &[
            "The extra large button's size: its container's height and shapes (round, square, pressed and selected), its \
             leading and trailing space, its icon's size and the space between icon and label.",
        ],
    },
    Source {
        file: "ButtonXSmallTokens.kt",
        object: None,
        doc: &[
            "The extra small button's size: its container's height and shapes (round, square, pressed and selected), its \
             leading and trailing space, its icon's size and the space between icon and label.",
        ],
    },
    Source {
        file: "CheckboxTokens.kt",
        object: None,
        doc: &[
            "The checkbox's box (its size, corner shape and outline widths), its mark's size, its state layer's size \
             and shape, and its colors in each state, selected and unselected, enabled, disabled and with an error. \
             Colors are roles of the scheme ([`crate::color::ColorRole`]), resolved against the light or dark \
             [`crate::color::ColorScheme`].",
        ],
    },
    Source {
        file: "CircularProgressIndicatorTokens.kt",
        object: None,
        doc: &[
            "The circular progress indicator's sizes: its diameter and stroke widths, and the wave's amplitude, \
             wavelength and diameter for the wavy one.",
        ],
    },
    Source {
        file: "ColorDarkTokens.kt",
        object: None,
        doc: &[
            "The baseline dark color scheme: each color role as a tone of [`super::palette_tokens`].",
        ],
    },
    Source {
        file: "ColorLightTokens.kt",
        object: None,
        doc: &[
            "The baseline light color scheme: each color role as a tone of [`super::palette_tokens`].",
        ],
    },
    Source {
        file: "DividerTokens.kt",
        object: None,
        doc: &[
            "The divider's color and thickness. The color is a role of the scheme ([`crate::color::ColorRole`]), \
             resolved against the light or dark [`crate::color::ColorScheme`].",
        ],
    },
    Source {
        file: "ElevatedButtonTokens.kt",
        object: None,
        doc: &[
            "The elevated button's colors and elevation in each state. Colors are roles of the scheme \
             ([`crate::color::ColorRole`]), resolved against the light or dark [`crate::color::ColorScheme`].",
        ],
    },
    Source {
        file: "ElevationTokens.kt",
        object: None,
        doc: &[
            "The elevation levels 0 to 5: how far a surface sits above the one under it. androidx gives them in dp, \
             the density-independent pixel, which is NSH's point.",
        ],
    },
    Source {
        file: "ExpressiveMotionTokens.kt",
        object: None,
        doc: &[
            "The springs of the Expressive motion scheme: a stiffness and a damping *ratio* each, at unit mass. NSH's \
             `Spring` takes a damping *coefficient*: use these through [`crate::motion::SpringToken`], which \
             converts them (ADR 0004).",
        ],
    },
    Source {
        file: "FabBaselineTokens.kt",
        object: None,
        doc: &[
            "The floating action button's size: its container's height and width, its shape and its icon's size.",
        ],
    },
    Source {
        file: "FabLargeTokens.kt",
        object: None,
        doc: &[
            "The large floating action button's size: its container's height and width, its shape and its icon's \
             size.",
            "androidx's `FloatingActionButton.kt` does not read `ICON_SIZE`: it writes 36 in \
             `FloatingActionButtonDefaults.LargeIconSize`, under \"FabLargeTokens.IconSize is incorrect\" (ADR 0012).",
        ],
    },
    Source {
        file: "FabMediumTokens.kt",
        object: None,
        doc: &[
            "The medium floating action button's size: its container's height and width and its icon's size.",
            "The file has no container shape: androidx comments it out (\"TODO: uncomment when \
             ShapeKeyTokens.CornerLargeIncreased is available\"), and `FloatingActionButton.kt` takes \
             `ShapeDefaults.LargeIncreased` instead (ADR 0012).",
        ],
    },
    Source {
        file: "FabPrimaryContainerTokens.kt",
        object: None,
        doc: &[
            "The primary container floating action button's container and icon colors, and its container's \
             elevation, in each state. Colors are roles of the scheme ([`crate::color::ColorRole`]), resolved \
             against the light or dark [`crate::color::ColorScheme`].",
        ],
    },
    Source {
        file: "FabSecondaryContainerTokens.kt",
        object: None,
        doc: &[
            "The secondary container floating action button's container and icon colors, and its container's \
             elevation, in each state. Colors are roles of the scheme ([`crate::color::ColorRole`]), resolved \
             against the light or dark [`crate::color::ColorScheme`].",
        ],
    },
    Source {
        file: "FabSmallTokens.kt",
        object: None,
        doc: &[
            "The small floating action button's size: its container's height and width, its shape and its icon's \
             size.",
        ],
    },
    Source {
        file: "FilledButtonTokens.kt",
        object: None,
        doc: &[
            "The filled button's colors and elevation in each state. Colors are roles of the scheme \
             ([`crate::color::ColorRole`]), resolved against the light or dark [`crate::color::ColorScheme`].",
        ],
    },
    Source {
        file: "FilledIconButtonTokens.kt",
        object: None,
        doc: &[
            "The filled icon button's container and icon colors in each state, and those of the filled icon toggle button selected and \
             unselected. Colors are roles of the scheme ([`crate::color::ColorRole`]), resolved against the light or \
             dark [`crate::color::ColorScheme`].",
        ],
    },
    Source {
        file: "FilledTonalButtonTokens.kt",
        object: None,
        doc: &[
            "The tonal button's colors, elevation and label style in each state. androidx's `Button.kt` reads this \
             file for the tonal button, not [`super::tonal_button_tokens`] (ADR 0012).",
            "Colors are roles of the scheme ([`crate::color::ColorRole`]), resolved against the light or dark \
             [`crate::color::ColorScheme`].",
        ],
    },
    Source {
        file: "FilledTonalIconButtonTokens.kt",
        object: None,
        doc: &[
            "The tonal icon button's container and icon colors in each state, and those of the tonal icon toggle button selected and \
             unselected. Colors are roles of the scheme ([`crate::color::ColorRole`]), resolved against the light or \
             dark [`crate::color::ColorScheme`].",
        ],
    },
    Source {
        file: "IconButtonTokens.kt",
        object: Some("StandardIconButtonTokens"),
        doc: &[
            "The standard icon button's icon colors (it has no container) in each state, and those of the standard icon toggle button selected and \
             unselected. Colors are roles of the scheme ([`crate::color::ColorRole`]), resolved against the light or \
             dark [`crate::color::ColorScheme`].",
        ],
    },
    Source {
        file: "LargeIconButtonTokens.kt",
        object: None,
        doc: &[
            "The large icon button's size: its container's height, its icon's size, its leading and trailing \
             space for each width (narrow, default and wide), its shapes (round, square, pressed and selected) and \
             its outline's width.",
        ],
    },
    Source {
        file: "LinearProgressIndicatorTokens.kt",
        object: None,
        doc: &[
            "The linear progress indicator's sizes: its height, the stop and gap sizes, and the wave's amplitude, \
             wavelength and height for the wavy one.",
        ],
    },
    Source {
        file: "MediumIconButtonTokens.kt",
        object: None,
        doc: &[
            "The medium icon button's size: its container's height, its icon's size, its leading and trailing \
             space for each width (narrow, default and wide), its shapes (round, square, pressed and selected) and \
             its outline's width.",
        ],
    },
    Source {
        file: "MotionTokens.kt",
        object: None,
        doc: &[
            "Durations, in milliseconds (the unit of NSH's `Timing::duration_ms`), and easing curves.",
        ],
    },
    Source {
        file: "OutlinedButtonTokens.kt",
        object: None,
        doc: &[
            "The outlined button's colors and elevation in each state. Colors are roles of the scheme \
             ([`crate::color::ColorRole`]), resolved against the light or dark [`crate::color::ColorScheme`].",
        ],
    },
    Source {
        file: "OutlinedIconButtonTokens.kt",
        object: None,
        doc: &[
            "The outlined icon button's icon and outline colors in each state, and those of the outlined icon toggle button selected and \
             unselected. Colors are roles of the scheme ([`crate::color::ColorRole`]), resolved against the light or \
             dark [`crate::color::ColorScheme`].",
        ],
    },
    Source {
        file: "PaletteTokens.kt",
        object: None,
        doc: &[
            "The baseline tonal palettes: each key color (primary, secondary, tertiary, neutral, neutral variant, \
             error) at a range of tones from 0 (black) to 100 (white), in 8-bit sRGB.",
            "Components never draw with a palette color: they draw with a color role of the scheme, which names one \
             of these tones ([`super::color_light_tokens`], [`super::color_dark_tokens`]).",
        ],
    },
    Source {
        file: "ProgressIndicatorTokens.kt",
        object: None,
        doc: &[
            "The colors and shapes the linear and circular progress indicators share: the active indicator, the \
             stop and the track. Colors are roles of the scheme ([`crate::color::ColorRole`]).",
        ],
    },
    Source {
        file: "ShapeTokens.kt",
        object: None,
        doc: &[
            "The corner shapes, from square (`CORNER_NONE`) to fully round (`CORNER_FULL`), and the corner sizes \
             alone (`CORNER_VALUE_*`), in dp, which is NSH's point.",
            "Components name a shape through an androidx `ShapeKeyTokens` key, which has the name of one of the \
             shapes here: `ShapeKeyTokens.CornerFull` is [`CORNER_FULL`].",
        ],
    },
    Source {
        file: "SmallIconButtonTokens.kt",
        object: None,
        doc: &[
            "The small icon button's size: its container's height, its icon's size, its leading and trailing \
             space for each width (narrow, default and wide), its shapes (round, square, pressed and selected) and \
             its outline's width.",
        ],
    },
    Source {
        file: "SnackbarTokens.kt",
        object: None,
        doc: &[
            "The snackbar: its container's color, shape and elevation, the colors and type styles of its supporting \
             text, its action's label and its icon, and its heights with one and two lines of text. Colors are \
             roles of the scheme ([`crate::color::ColorRole`]).",
        ],
    },
    Source {
        file: "StateTokens.kt",
        object: None,
        doc: &[
            "The opacities of the state layer: a layer of the content's color drawn over a component while it is \
             hovered, focused, pressed or dragged.",
        ],
    },
    Source {
        file: "SwitchTokens.kt",
        object: None,
        doc: &[
            "The switch's track, handle and icon: their sizes, and their colors in each state, selected and unselected, \
             enabled and disabled. Colors are roles of the scheme ([`crate::color::ColorRole`]), resolved against the \
             light or dark [`crate::color::ColorScheme`].",
        ],
    },
    Source {
        file: "TextButtonTokens.kt",
        object: None,
        doc: &[
            "The text button's colors and elevation in each state. Colors are roles of the scheme \
             ([`crate::color::ColorRole`]), resolved against the light or dark [`crate::color::ColorScheme`].",
        ],
    },
    Source {
        file: "TonalButtonTokens.kt",
        object: None,
        doc: &[
            "The tonal toggle button's colors and elevation in each state: androidx reads this file only in \
             `ToggleButton.kt`. The tonal button reads [`super::filled_tonal_button_tokens`] (ADR 0012).",
            "Colors are roles of the scheme ([`crate::color::ColorRole`]), resolved against the light or dark \
             [`crate::color::ColorScheme`].",
        ],
    },
    Source {
        file: "TypeScaleTokens.kt",
        object: None,
        doc: &[
            "The type scale: for each style (display, headline, title, body and label, each large, medium and \
             small, and each also emphasized), its font, weight, size, line height and tracking.",
            "Sizes, line heights and tracking are in sp, the scaled pixel, as NSH points. NSH scales a text's size \
             and line height by the user's text size, as Android scales sp; it does not scale tracking.",
        ],
    },
    Source {
        file: "TypefaceTokens.kt",
        object: None,
        doc: &[
            "The typefaces of the type scale (brand and plain) and its three weights (regular, medium and bold).",
        ],
    },
    Source {
        file: "TypographyTokens.kt",
        object: None,
        doc: &[
            "The type styles: each style of the type scale as one value, with its font, weight, size, line height and \
             tracking from [`super::type_scale_tokens`].",
            "androidx's styles also center each line's text in its line height and trim nothing \
             (`DefaultLineHeightStyle`), and let an app give `Typography` a font family of its own; the library \
             keeps the token's family.",
        ],
    },
    Source {
        file: "XLargeIconButtonTokens.kt",
        object: None,
        doc: &[
            "The extra large icon button's size: its container's height, its icon's size, its leading and trailing \
             space for each width (narrow, default and wide), its shapes (round, square, pressed and selected) and \
             its outline's width.",
        ],
    },
    Source {
        file: "XSmallIconButtonTokens.kt",
        object: None,
        doc: &[
            "The extra small icon button's size: its container's height, its icon's size, its leading and trailing \
             space for each width (narrow, default and wide), its shapes (round, square, pressed and selected) and \
             its outline's width.",
        ],
    },
];

/// Vendored files of keys (`val Primary = ColorToken(25)`) that component tokens name, read to check those names
/// but not turned into modules: the library has its own type for each kind of key.
const KEY_SOURCES: &[&str] = &[
    "ColorSchemeKeyTokens.kt",
    "ShapeKeyTokens.kt",
    "TypographyKeyTokens.kt",
];

const INDEX_DOC: &[&str] = &[
    "The Material 3 tokens, generated from the androidx Kotlin files vendored in \
     `reference/androidx-material3-tokens/` (ADR 0003).",
    "Each constant names its androidx token in its documentation. Numbers are `f32`, the type NSH takes; the \
     generator refuses a value that `f32` cannot hold exactly, except a text unit (`0.1.sp`) or a length \
     (`1.6.dp`), which androidx itself rounds to the nearest `f32`.",
];

const REFERENCE_DIR: &str = "reference/androidx-material3-tokens";
const OUTPUT_DIR: &str = "crates/nsh-m3e/src/tokens/androidx";

const ICONS_REFERENCE_DIR: &str = "reference/material-symbols";
const ICONS_OUTPUT_DIR: &str = "crates/nsh-m3e/src/material_symbols";
/// The table that pins each vendored source, where the `material-symbols/` row names the icons' commit.
const REFERENCE_README: &str = "reference/README.md";

const USAGE: &str = "usage: cargo xtask gen-tokens [--check] | cargo xtask gen-icons [--check]";

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
        expected: String,
    },
    BadReference {
        path: PathBuf,
        token: String,
        kind: kotlin::Kind,
        reference: kotlin::Reference,
    },
    Svg {
        path: PathBuf,
        source: icons::SvgError,
    },
    IconSet {
        path: PathBuf,
        source: icons::SetError,
    },
    NoPin(PathBuf),
    Outdated {
        task: Task,
        paths: Vec<PathBuf>,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Usage(message) => write!(f, "{message}\n{USAGE}"),
            Error::NoWorkspaceRoot => write!(f, "cannot find the workspace root"),
            Error::Io { path, source } => write!(f, "{}: {source}", path.display()),
            Error::Parse { path, source } => write!(f, "{}: {source}", path.display()),
            Error::WrongObject {
                path,
                found,
                expected,
            } => write!(
                f,
                "{}: the file defines `{found}`, not `{expected}`",
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
            Error::Svg { path, source } => write!(f, "{}: {source}", path.display()),
            Error::IconSet { path, source } => write!(f, "{}: {source}", path.display()),
            Error::NoPin(path) => write!(
                f,
                "{}: no commit in the `material-symbols/` row",
                path.display()
            ),
            Error::Outdated { task, paths } => {
                let (what, command) = match task {
                    Task::Tokens => ("tokens", "gen-tokens"),
                    Task::Icons => ("icons", "gen-icons"),
                };
                writeln!(
                    f,
                    "the generated {what} are out of date; run `cargo xtask {command}`:"
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

/// What a run generates.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Task {
    Tokens,
    Icons,
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
    let (task, mode) = mode(args)?;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or(Error::NoWorkspaceRoot)?;
    match task {
        Task::Tokens => gen_tokens(root, mode),
        Task::Icons => gen_icons(root, mode),
    }
}

fn mode(args: &[String]) -> Result<(Task, Mode), Error> {
    let task = |name: &str| match name {
        "gen-tokens" => Some(Task::Tokens),
        "gen-icons" => Some(Task::Icons),
        _ => None,
    };
    match args {
        [name] => task(name).map(|task| (task, Mode::Write)),
        [name, flag] if flag == "--check" => task(name).map(|task| (task, Mode::Check)),
        _ => None,
    }
    .ok_or_else(|| Error::Usage(format!("unknown arguments {args:?}")))
}

/// Reads and parses one vendored file, whose object must have the file's name.
/// Reads the object of `file`, which must be named `object`, or after the file when `object` is `None`.
fn read_object(
    root: &Path,
    file: &str,
    object: Option<&str>,
) -> Result<(PathBuf, kotlin::TokenObject), Error> {
    let path = root.join(REFERENCE_DIR).join(file);
    let text = read(&path)?;
    let parsed = kotlin::parse(&text).map_err(|source| Error::Parse {
        path: path.clone(),
        source,
    })?;
    let expected = object.unwrap_or_else(|| file.strip_suffix(".kt").unwrap_or(file));
    if parsed.name != expected {
        return Err(Error::WrongObject {
            path,
            found: parsed.name,
            expected: expected.to_owned(),
        });
    }
    Ok((path, parsed))
}

/// Generates every module in memory, then writes or compares them all, so a failure leaves no half-written state.
fn gen_tokens(root: &Path, mode: Mode) -> Result<(), Error> {
    let output_dir = root.join(OUTPUT_DIR);
    let mut objects = Vec::with_capacity(SOURCES.len() + KEY_SOURCES.len());
    for source in SOURCES {
        objects.push(read_object(root, source.file, source.object)?);
    }
    // The key files come last: references are checked against them, and the zip below, which stops with
    // `SOURCES`, never makes a module of them.
    for file in KEY_SOURCES {
        objects.push(read_object(root, file, None)?);
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

    let existing = existing_files(&output_dir)?;
    sync(files, existing, Task::Tokens, mode)
}

/// Generates every icon module in memory from the vendored SVG files, then writes or compares them all.
fn gen_icons(root: &Path, mode: Mode) -> Result<(), Error> {
    let readme = root.join(REFERENCE_README);
    let revision = pinned_icons_revision(&read(&readme)?).ok_or(Error::NoPin(readme))?;
    let output_dir = root.join(ICONS_OUTPUT_DIR);
    let mut files = vec![(output_dir.join("mod.rs"), icons::index(&revision))];
    for style_name in icons::STYLES {
        let dir = root.join(ICONS_REFERENCE_DIR).join(style_name);
        let mut svgs = Vec::new();
        for path in existing_files(&dir)? {
            let file = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default()
                .to_owned();
            let svg = icons::parse(&read(&path)?).map_err(|source| Error::Svg {
                path: path.clone(),
                source,
            })?;
            svgs.push((file, svg));
        }
        let style = icons::pair(svgs).map_err(|source| Error::IconSet {
            path: dir.clone(),
            source,
        })?;
        let module_dir = output_dir.join(style_name);
        files.push((
            module_dir.join("mod.rs"),
            icons::module(style_name, &style, false),
        ));
        files.push((
            module_dir.join("filled.rs"),
            icons::module(style_name, &style, true),
        ));
    }
    let mut existing = Vec::new();
    for path in existing_files(&output_dir)? {
        if path.is_dir() {
            existing.extend(existing_files(&path)?);
        } else {
            existing.push(path);
        }
    }
    sync(files, existing, Task::Icons, mode)
}

/// The commit of the `material-symbols/` row of `reference/README.md`, the one `scripts/sync-material-symbols.sh`
/// pins.
fn pinned_icons_revision(readme: &str) -> Option<String> {
    let row = readme
        .lines()
        .find(|line| line.starts_with("| `material-symbols/` |"))?;
    row.split('|')
        .map(str::trim)
        .filter_map(|cell| cell.strip_prefix('`')?.strip_suffix('`'))
        .find(|cell| cell.len() == 40 && cell.bytes().all(|b| b.is_ascii_hexdigit()))
        .map(str::to_owned)
}

/// Writes `files`, removing the `existing` ones it does not hold, or, to check, fails unless they are all as written
/// and nothing else is there.
fn sync(
    files: Vec<(PathBuf, String)>,
    existing: Vec<PathBuf>,
    task: Task,
    mode: Mode,
) -> Result<(), Error> {
    let expected: BTreeSet<PathBuf> = files.iter().map(|(path, _)| path.clone()).collect();
    let stale = existing.into_iter().filter(|path| !expected.contains(path));
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
                Err(Error::Outdated {
                    task,
                    paths: outdated,
                })
            }
        }
        Mode::Write => {
            for path in stale {
                fs::remove_file(&path).map_err(|source| Error::Io { path, source })?;
            }
            for (path, content) in files {
                if let Some(dir) = path.parent() {
                    fs::create_dir_all(dir).map_err(|source| Error::Io {
                        path: dir.to_owned(),
                        source,
                    })?;
                }
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
            // A shape key names the shape of the same name in `ShapeTokens`, and a typography key the text style of
            // the same name in `TypographyTokens`, which must exist too (androidx `Shapes.fromToken`,
            // `Typography.fromToken`).
            let namesake_found = match kind {
                kotlin::Kind::ShapeKey => find(&kotlin::Reference {
                    object: "ShapeTokens".into(),
                    token: reference.token.clone(),
                })
                .is_some_and(|shape| matches!(shape.value, kotlin::Value::Shape(_))),
                kotlin::Kind::TypographyKey => find(&kotlin::Reference {
                    object: "TypographyTokens".into(),
                    token: reference.token.clone(),
                })
                .is_some_and(|style| matches!(style.value, kotlin::Value::TextStyle(_))),
                _ => true,
            };
            if !namesake_found || !target.is_some_and(|target| kind.holds(&target.value)) {
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
            &["gen-icons", "--force"],
            &["--check", "gen-icons"],
        ] {
            assert!(matches!(mode(&args(bad)), Err(Error::Usage(_))), "{bad:?}");
        }
        assert!(matches!(run(&args(&["gen"])), Err(Error::Usage(_))));
        assert_eq!(
            mode(&args(&["gen-tokens"])).unwrap(),
            (Task::Tokens, Mode::Write)
        );
        assert_eq!(
            mode(&args(&["gen-tokens", "--check"])).unwrap(),
            (Task::Tokens, Mode::Check)
        );
        assert_eq!(
            mode(&args(&["gen-icons"])).unwrap(),
            (Task::Icons, Mode::Write)
        );
        assert_eq!(
            mode(&args(&["gen-icons", "--check"])).unwrap(),
            (Task::Icons, Mode::Check)
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
            Err(Error::Outdated {
                task: Task::Tokens,
                paths,
            }) => paths,
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
    fn a_typography_key_must_name_a_key_and_a_text_style_of_the_same_name() {
        // Keys `X`, which `typography` holds a style of, and `Y`, which it does not.
        const KEYS: &str = "@JvmInline internal value class TypographyToken(val id: Int)\ninternal object TypographyKeyTokens {\n    val X = TypographyToken(9)\n    val Y = TypographyToken(11)\n}\n";
        let key = |target| button("TypographyToken", target);
        let tokens = |keys: &str, target| {
            vec![
                type_scale(kotlin::Value::Sp(16.0)),
                typography(STYLE),
                object(keys),
                key(target),
            ]
        };
        check_references(&tokens(KEYS, "TypographyKeyTokens.X")).unwrap();
        // A key the file does not have, and a key with no style.
        assert_eq!(
            bad_kind(&tokens(KEYS, "TypographyKeyTokens.Z")),
            kotlin::Kind::TypographyKey
        );
        assert_eq!(
            bad_kind(&tokens(KEYS, "TypographyKeyTokens.Y")),
            kotlin::Kind::TypographyKey
        );
        // Keys of another class under the same object name are not typography keys.
        let shape_keys = KEYS.replace("TypographyToken", "ShapeToken");
        assert_eq!(
            bad_kind(&tokens(&shape_keys, "TypographyKeyTokens.X")),
            kotlin::Kind::TypographyKey
        );
        // No file of keys, no `TypographyTokens`, and a namesake that is not a text style.
        assert_eq!(
            bad_kind(&[
                type_scale(kotlin::Value::Sp(16.0)),
                typography(STYLE),
                key("TypographyKeyTokens.X")
            ]),
            kotlin::Kind::TypographyKey
        );
        assert_eq!(
            bad_kind(&[object(KEYS), key("TypographyKeyTokens.X")]),
            kotlin::Kind::TypographyKey
        );
        let not_a_style = kotlin::TokenObject {
            name: "TypographyTokens".into(),
            version: None,
            tokens: vec![kotlin::Token {
                name: "X".into(),
                value: kotlin::Value::Number(1.0),
            }],
        };
        assert_eq!(
            bad_kind(&[
                (PathBuf::from("TypographyTokens.kt"), not_a_style),
                object(KEYS),
                key("TypographyKeyTokens.X")
            ]),
            kotlin::Kind::TypographyKey
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
    fn a_source_named_for_another_object_must_define_that_object() {
        let scratch = Scratch::new("object");
        let root = scratch.0.as_path();

        // `IconButtonTokens.kt` defines `StandardIconButtonTokens`: read as named, it is accepted, and only then.
        let (_, object) = read_object(
            root,
            "IconButtonTokens.kt",
            Some("StandardIconButtonTokens"),
        )
        .unwrap();
        assert_eq!(object.name, "StandardIconButtonTokens");
        assert!(matches!(
            read_object(root, "IconButtonTokens.kt", None),
            Err(Error::WrongObject { found, expected, .. })
                if found == "StandardIconButtonTokens" && expected == "IconButtonTokens"
        ));
        assert!(matches!(
            read_object(root, "IconButtonTokens.kt", Some("FilledIconButtonTokens")),
            Err(Error::WrongObject { found, expected, .. })
                if found == "StandardIconButtonTokens" && expected == "FilledIconButtonTokens"
        ));

        // Its module is named after the object, not the file.
        gen_tokens(root, Mode::Write).unwrap();
        let output = root.join(OUTPUT_DIR);
        assert!(output.join("standard_icon_button_tokens.rs").exists());
        assert!(!output.join("icon_button_tokens.rs").exists());
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
        let outdated = Error::Outdated {
            task: Task::Tokens,
            paths: vec![PathBuf::from("a.rs"), PathBuf::from("b.rs")],
        };
        assert_eq!(
            outdated.to_string(),
            "the generated tokens are out of date; run `cargo xtask gen-tokens`:\n  a.rs\n  b.rs\n"
        );
        let outdated_icons = Error::Outdated {
            task: Task::Icons,
            paths: vec![PathBuf::from("mod.rs")],
        };
        assert_eq!(
            outdated_icons.to_string(),
            "the generated icons are out of date; run `cargo xtask gen-icons`:\n  mod.rs\n"
        );
        let svg = Error::Svg {
            path: PathBuf::from("a_24px.svg"),
            source: icons::SvgError::EmptyPath,
        };
        assert_eq!(svg.to_string(), "a_24px.svg: the path data is empty");
        let set = Error::IconSet {
            path: PathBuf::from("outlined"),
            source: icons::SetError::Unpaired("a".into()),
        };
        assert_eq!(
            set.to_string(),
            "outlined: `a` needs both `a_24px.svg` and `a_fill1_24px.svg`"
        );
        assert_eq!(
            Error::NoPin(PathBuf::from("README.md")).to_string(),
            "README.md: no commit in the `material-symbols/` row"
        );
        let wrong = Error::WrongObject {
            path: PathBuf::from("A.kt"),
            found: "B".into(),
            expected: "A".into(),
        };
        assert_eq!(wrong.to_string(), "A.kt: the file defines `B`, not `A`");
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
            let named_for_file = source.file.strip_suffix(".kt");
            assert_eq!(
                Some(object.name.as_str()),
                source.object.or(named_for_file),
                "{}",
                source.file
            );
            // An object is named only where the file's name does not say it.
            assert_ne!(source.object, named_for_file, "{}", source.file);
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

    const PINNED_ROW: &str = "| `material-symbols/` | https://github.com/google/material-design-icons | \
                              `10ecc14790612ca8fd164e3f4ffe9c16fc50d5b7` | Apache-2.0 |";

    #[test]
    fn the_icons_revision_is_the_commit_of_their_row() {
        let readme = format!("| Folder | Source | Revision |\n|---|---|---|\n{PINNED_ROW}\n");
        assert_eq!(
            pinned_icons_revision(&readme).as_deref(),
            Some("10ecc14790612ca8fd164e3f4ffe9c16fc50d5b7")
        );
        // Another row's commit, or a cell that is not a full commit, is not the pin.
        let other = PINNED_ROW.replace("material-symbols/", "androidx-material3-tokens/");
        assert_eq!(pinned_icons_revision(&other), None);
        let short = PINNED_ROW.replace("10ecc14790612ca8fd164e3f4ffe9c16fc50d5b7", "10ecc147");
        assert_eq!(pinned_icons_revision(&short), None);
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        assert!(pinned_icons_revision(&read(&root.join(REFERENCE_README)).unwrap()).is_some());
    }

    fn outdated_icons(result: Result<(), Error>) -> Vec<PathBuf> {
        match result {
            Err(Error::Outdated {
                task: Task::Icons,
                paths,
            }) => paths,
            other => panic!("expected Outdated icons, got {other:?}"),
        }
    }

    /// A throwaway workspace with two icons in each style.
    fn icon_scratch(name: &str) -> Scratch {
        let dir =
            std::env::temp_dir().join(format!("nsh-m3e-xtask-icons-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(REFERENCE_README), format!("{PINNED_ROW}\n")).ok();
        fs::create_dir_all(dir.join("reference")).unwrap();
        fs::write(dir.join(REFERENCE_README), format!("{PINNED_ROW}\n")).unwrap();
        for style in icons::STYLES {
            let style_dir = dir.join(ICONS_REFERENCE_DIR).join(style);
            fs::create_dir_all(&style_dir).unwrap();
            for (file, path) in [
                ("add_24px.svg", "M1 1Z"),
                ("add_fill1_24px.svg", "M2 2Z"),
                ("10k_24px.svg", "M3 3Z"),
                ("10k_fill1_24px.svg", "M4 4Z"),
            ] {
                fs::write(
                    style_dir.join(file),
                    format!(
                        "<svg xmlns=\"http://www.w3.org/2000/svg\" height=\"24\" viewBox=\"0 -960 960 960\" \
                         width=\"24\"><path d=\"{path}\"/></svg>"
                    ),
                )
                .unwrap();
            }
        }
        Scratch(dir)
    }

    #[test]
    fn the_icons_check_passes_after_generating_and_fails_on_any_drift() {
        let scratch = icon_scratch("drift");
        let root = scratch.0.as_path();
        let output = root.join(ICONS_OUTPUT_DIR);

        // Nothing generated yet: the index and two modules per style are missing.
        assert_eq!(
            outdated_icons(gen_icons(root, Mode::Check)).len(),
            1 + 2 * icons::STYLES.len()
        );
        gen_icons(root, Mode::Write).unwrap();
        gen_icons(root, Mode::Check).unwrap();
        let filled = fs::read_to_string(output.join("sharp").join("filled.rs")).unwrap();
        assert!(filled.contains("pub const _10K: Icon = Icon::from_static(\"M4 4Z\""));

        // A hand edit and a stray file are caught; writing again restores the one and removes the other.
        let edited = output.join("rounded").join("mod.rs");
        let text = fs::read_to_string(&edited).unwrap();
        fs::write(&edited, text.replace("M1 1Z", "M9 9Z")).unwrap();
        let stray = output.join("outlined").join("stray.rs");
        fs::write(&stray, "").unwrap();
        let mut drift = outdated_icons(gen_icons(root, Mode::Check));
        drift.sort();
        assert_eq!(drift, vec![stray.clone(), edited]);
        gen_icons(root, Mode::Write).unwrap();
        assert!(!stray.exists());
        gen_icons(root, Mode::Check).unwrap();

        // A change to a vendored file is caught too.
        let svg = root
            .join(ICONS_REFERENCE_DIR)
            .join("outlined")
            .join("add_24px.svg");
        let text = fs::read_to_string(&svg).unwrap();
        fs::write(&svg, text.replace("M1 1Z", "M5 5Z")).unwrap();
        assert_eq!(
            outdated_icons(gen_icons(root, Mode::Check)),
            vec![output.join("outlined").join("mod.rs")]
        );
    }

    #[test]
    fn a_vendored_icon_that_is_unpaired_or_unknown_stops_the_generation() {
        let scratch = icon_scratch("bad");
        let root = scratch.0.as_path();
        let style_dir = root.join(ICONS_REFERENCE_DIR).join("rounded");
        fs::remove_file(style_dir.join("add_fill1_24px.svg")).unwrap();
        assert!(matches!(
            gen_icons(root, Mode::Write),
            Err(Error::IconSet { source: icons::SetError::Unpaired(name), .. }) if name == "add"
        ));
        fs::write(style_dir.join("add_fill1_24px.svg"), "<svg/>").unwrap();
        assert!(matches!(
            gen_icons(root, Mode::Write),
            Err(Error::Svg {
                source: icons::SvgError::Form,
                ..
            })
        ));
        // Nothing was written.
        assert!(!root.join(ICONS_OUTPUT_DIR).exists());
        fs::write(root.join(REFERENCE_README), "").unwrap();
        assert!(matches!(gen_icons(root, Mode::Write), Err(Error::NoPin(_))));
    }

    #[test]
    fn every_vendored_material_symbol_parses_in_pairs() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        for style in icons::STYLES {
            let dir = root.join(ICONS_REFERENCE_DIR).join(style);
            let files = existing_files(&dir)
                .unwrap()
                .into_iter()
                .map(|path| {
                    let file = path.file_name().unwrap().to_str().unwrap().to_owned();
                    (file, icons::parse(&read(&path).unwrap()).unwrap())
                })
                .collect();
            // google/material-design-icons at the pinned commit has 4,157 icons in each style.
            assert_eq!(icons::pair(files).unwrap().len(), 4157, "{style}");
        }
    }
}
