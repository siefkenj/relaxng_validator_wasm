use clap::Parser;
use relaxng_validator_wasm::{VirtualFileSystem, check_simple};
use std::path::PathBuf;
use std::process;

/// Validate an XML document against RelaxNG grammars.
///
/// The first one or more files are grammar files (`.rnc` or `.rng`).
/// The first grammar file is the root grammar. The final file is the input XML
/// document to validate.
#[derive(Parser)]
#[command(
    name = "relaxng-validator",
    version,
    about = "Validate XML against RelaxNG grammars",
    arg_required_else_help = true,
    long_about = "Validate XML against RelaxNG grammars.\n\nArgument order matters:\n- First one or more files: RelaxNG grammar files (.rnc or .rng)\n- Last file: input XML document\n\nThe first grammar file is used as the root grammar. Additional grammar files are loaded into an in-memory virtual file system for includes/external references.",
    after_help = "Examples:\n  relaxng-validator main.rnc input.xml\n  relaxng-validator main.rnc chapter.rnc input.xml"
)]
struct Args {
    /// First files: grammar files (.rnc/.rng). Last file: input XML document.
    #[arg(required = true, num_args = 2.., value_name = "FILES")]
    files: Vec<PathBuf>,
}

fn main() {
    let args = Args::parse();

    let (schema_files, xml_file) = args.files.split_at(args.files.len() - 1);
    let xml_path = &xml_file[0];
    let root_schema_name = schema_files[0]
        .file_name()
        .expect("schema path has no file name")
        .to_string_lossy()
        .into_owned();

    // Build the virtual file system from all schema files.
    let mut vfs_map = std::collections::HashMap::new();
    for path in schema_files {
        let name = path
            .file_name()
            .expect("schema path has no file name")
            .to_string_lossy()
            .into_owned();
        let content = std::fs::read_to_string(path).unwrap_or_else(|e| {
            eprintln!("error: could not read '{}': {e}", path.display());
            process::exit(1);
        });
        vfs_map.insert(name, content);
    }
    let vfs = VirtualFileSystem::from_map(vfs_map);

    let doc = std::fs::read_to_string(xml_path).unwrap_or_else(|e| {
        eprintln!("error: could not read '{}': {e}", xml_path.display());
        process::exit(1);
    });

    match check_simple(vfs, &root_schema_name, &doc) {
        Ok(()) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({ "errors": [] })).unwrap()
            );
        }
        Err(errors) => {
            eprintln!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({ "errors": errors })).unwrap()
            );
            process::exit(1);
        }
    }
}
