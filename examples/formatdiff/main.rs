mod diff;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Parser;
use diff::FieldDiff;
use plwrapper::{PatternLang, PlField, PlResult};

const EXTENSION_ALIASES: &[(&str, &str)] = &[
    ("jpg", "jpeg"),
    ("tif", "tiff"),
    ("htm", "html"),
    ("yml", "yaml"),
];

const MAX_RAW_DIFF_RANGES: usize = 50;
const DIFF_CONTEXT: usize = 3;

#[derive(Parser)]
#[command(
    name = "formatdiff",
    about = "Parse and compare binary formats field-by-field tools",
    version = env!("CARGO_PKG_VERSION"),
    long_version = concat!(
        env!("CARGO_PKG_VERSION"),
        "\n\n",
        include_str!("../../ThirdPartyNotices.txt")
    )
)]
struct Cli {
    file_a: PathBuf,

    // 省略時はfile_aのフィールドをすべて表示
    // パイプにつないでdiffutilsでの比較を想定
    file_b: Option<PathBuf>,

    // 省略時は拡張子からpatternを参照
    #[arg(long, short = 'f', conflicts_with = "pattern")]
    format: Option<String>,

    #[arg(long, short = 'p', conflicts_with = "format")]
    pattern: Option<PathBuf>,

    #[arg(long = "include", short = 'I')]
    includes: Vec<PathBuf>,

    #[arg(long = "patterns-root", default_value = ".")]
    patterns_root: PathBuf,

    #[arg(long)]
    only_diff: bool,
}

struct ResolvedPattern {
    path: PathBuf,
    include_dirs: Vec<PathBuf>,
}

fn resolve_pattern(cli: &Cli) -> Result<ResolvedPattern, String> {
    if let Some(pattern) = &cli.pattern {
        return Ok(ResolvedPattern {
            path: pattern.clone(),
            include_dirs: Vec::new(),
        });
    }

    let format = match &cli.format {
        Some(format) => format.clone(),
        None => {
            let ext = cli
            .file_a
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| {
                "The file has no extension, The format can't be auto-detected. Specify --format or --pattern."
                .to_string()
            })?
            .to_lowercase();

            EXTENSION_ALIASES
                .iter()
                .find(|(from, _)| *from == ext)
                .map(|(_, to)| to.to_string())
                .unwrap_or(ext)
        }
    };

    let root = &cli.patterns_root;
    let path = root.join(format!("{format}.hexpat"));

    if !path.is_file() {
        return Err(format!(
            "No pattern file found for format '{format}': {}\n\
             Try a different name with --format, specify a .hexpat file directly with --pattern, \
             or point --patterns-root at a directory containing {format}.hexpat \
             (currently: {}).",
            path.display(),
            root.display()
        ));
    }

    Ok(ResolvedPattern {
        path,
        include_dirs: vec![root.clone()],
    })
}

fn analyze(pattern: &ResolvedPattern, cli: &Cli, data_path: &Path) -> Result<PlResult, String> {
    let mut runtime = PatternLang::new();
    for dir in pattern.include_dirs.iter().chain(cli.includes.iter()) {
        runtime.add_include_path(dir);
    }

    let result = runtime
        .run(&pattern.path, data_path)
        .map_err(|message| format!("Failed to parse {}: {message}", data_path.display()))?;

    if let Some(message) = &result.error {
        eprintln!(
            "Warning: parsing {} stopped early (using partial results): {message}",
            data_path.display()
        );
    }

    Ok(result)
}

fn read_file(path: &Path) -> Result<Vec<u8>, String> {
    std::fs::read(path).map_err(|err| format!("Can't read {}: {err}", path.display()))
}

fn field_summary(field: &PlField) -> String {
    format!(
        "{}: {} @ 0x{:X} (size={})",
        field.type_name, field.value, field.offset, field.size
    )
}

fn field_summary_with_bytes(field: &PlField, data: &[u8]) -> String {
    let base = field_summary(field);
    match diff::field_bytes(data, field) {
        Some(bytes) => {
            let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
            format!("{base} bytes={hex}")
        }
        None => format!("{base} bytes=<out-of-range>"),
    }
}

enum DiffLine<'a> {
    Context(&'a PlField),
    Removed(&'a PlField),
    Added(&'a PlField),
}

fn flatten_diff_lines<'a>(comparisons: &'a [FieldDiff<'a>]) -> Vec<DiffLine<'a>> {
    let mut lines = Vec::with_capacity(comparisons.len());

    for comparison in comparisons {
        match comparison {
            FieldDiff::Same(field) => lines.push(DiffLine::Context(field)),
            FieldDiff::Changed(a, b) => {
                lines.push(DiffLine::Removed(a));
                lines.push(DiffLine::Added(b));
            }
            FieldDiff::OnlyA(field) => lines.push(DiffLine::Removed(field)),
            FieldDiff::OnlyB(field) => lines.push(DiffLine::Added(field)),
        }
    }

    lines
}

fn diff_line_text(line: &DiffLine) -> String {
    let (marker, field) = match line {
        DiffLine::Context(field) => (' ', field),
        DiffLine::Removed(field) => ('-', field),
        DiffLine::Added(field) => ('+', field),
    };
    format!("{marker}{:<40} {}", field.path, field_summary(field))
}

fn print_unified_diff(
    comparisons: &[FieldDiff],
    file_a: &Path,
    file_b: &Path,
    context: usize,
) -> bool {
    let lines = flatten_diff_lines(comparisons);

    let change_indices: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| !matches!(line, DiffLine::Context(_)))
        .map(|(i, _)| i)
        .collect();

    if change_indices.is_empty() {
        return false;
    }

    let mut groups: Vec<(usize, usize)> = Vec::new();
    let mut cur_start = change_indices[0];
    let mut cur_end = change_indices[0];
    for &idx in &change_indices[1..] {
        if idx <= cur_end + 2 * context + 1 {
            cur_end = idx;
        } else {
            groups.push((cur_start, cur_end));
            cur_start = idx;
            cur_end = idx;
        }
    }
    groups.push((cur_start, cur_end));

    println!("--- {}", file_a.display());
    println!("+++ {}", file_b.display());

    for (start, end) in groups {
        let hunk_start = start.saturating_sub(context);
        let hunk_end = (end + context + 1).min(lines.len());

        for line in &lines[hunk_start..hunk_end] {
            println!("{}", diff_line_text(line));
        }
    }

    true
}

fn print_raw_fallback(data_a: &[u8], data_b: &[u8]) -> bool {
    let mut has_diff = false;

    println!("Fallback: Raw byte diff:");

    if data_a.len() != data_b.len() {
        has_diff = true;
        println!(
            "  Filesize: A={} bytes, B={} bytes",
            data_a.len(),
            data_b.len()
        );
    }

    let ranges = diff::raw_byte_diff_ranges(data_a, data_b);
    if ranges.is_empty() {
        println!(
            "  No difference in the first {} bytes.",
            data_a.len().min(data_b.len())
        );
    } else {
        has_diff = true;
        for (start, end) in ranges.iter().take(MAX_RAW_DIFF_RANGES) {
            println!(
                "  Diff range: 0x{start:X}-0x{end:X} ({} bytes)",
                end - start
            );
        }
        if ranges.len() > MAX_RAW_DIFF_RANGES {
            println!(
                "  ...and {} more diff range(s)",
                ranges.len() - MAX_RAW_DIFF_RANGES
            );
        }
    }

    has_diff
}

fn run_single(pattern: &ResolvedPattern, cli: &Cli, data_path: &Path) -> Result<(), String> {
    let data = read_file(data_path)?;
    let result = analyze(pattern, cli, data_path)?;

    eprintln!(
        "Pattern: {}\nFile: {}\n",
        pattern.path.display(),
        data_path.display()
    );

    for field in &result.fields {
        println!(
            "  {:<40} {}",
            field.path,
            field_summary_with_bytes(field, &data)
        );
    }

    Ok(())
}

fn run(cli: &Cli) -> Result<bool, String> {
    let pattern = resolve_pattern(cli)?;

    let Some(file_b) = &cli.file_b else {
        run_single(&pattern, cli, &cli.file_a)?;
        return Ok(false);
    };

    let data_a = read_file(&cli.file_a)?;
    let data_b = read_file(file_b)?;

    let result_a = analyze(&pattern, cli, &cli.file_a)?;
    let result_b = analyze(&pattern, cli, file_b)?;

    eprintln!("Pattern: {}\n", pattern.path.display());

    let comparisons = diff::diff_fields(&result_a.fields, &result_b.fields, &data_a, &data_b);
    let context = if cli.only_diff { 0 } else { DIFF_CONTEXT };
    let mut has_diff = print_unified_diff(&comparisons, &cli.file_a, file_b, context);

    if result_a.error.is_some() || result_b.error.is_some() {
        // has_diffがtrueであればfalseにしない
        has_diff |= print_raw_fallback(&data_a, &data_b);
    }

    Ok(has_diff)
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let single_file = cli.file_b.is_none();

    match run(&cli) {
        Ok(true) => ExitCode::from(1),
        Ok(false) => {
            if !single_file {
                println!("No diff.");
            }
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("Error: {message}");
            ExitCode::from(2)
        }
    }
}
