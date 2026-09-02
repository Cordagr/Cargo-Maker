//! Entry point for `cargo cmake-bridge`.

mod cli;
mod metadata;
mod template;
mod generator;

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err:#}");
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let args = cli::parse_args();

    let selected: Vec<Option<String>> = if args.all {
        metadata::list_crates_with_lib_target(&args.manifest_path)?
            .into_iter()
            .map(Some)
            .collect()
    } else if !args.crate_names.is_empty() {
        args.crate_names.iter().cloned().map(Some).collect()
    } else {
        vec![None]
    };

    for crate_name in selected {
        let crate_info = metadata::load(
            &args.manifest_path,
            args.target_dir.as_deref(),
            &args.profile,
            crate_name.as_deref(),
            args.generate_header,
        )?;
        let rendered = template::render(&crate_info);
        let crate_name_pascal = template::to_pascal_case(&crate_info.name);
        let out_path = generator::write_find_module(&args.out_dir, &crate_name_pascal, &rendered)?;
        println!("wrote {}", out_path.display());
    }
    Ok(())
}

