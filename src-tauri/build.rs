use std::env;
use std::fs;

fn make_aumid_rs()
{
	let metadata = cargo_metadata::MetadataCommand::new()
		.no_deps()
		.exec()
		.expect("Failed to read Cargo.toml metadata");

	let aumid = metadata
		.root_package()
		.and_then(|pkg| pkg.metadata.get("aumid").and_then(|v| v.as_str()))
		.expect("aumid not found in Cargo.toml");

	let out_dir = env::var("OUT_DIR").unwrap();
	let dest_path = format!("{}/aumid.rs", out_dir);
	fs::write(dest_path, format!("pub const AUMID : &str = \"{}\";", aumid))
		.expect("Failed to write AUMID file");
}

fn main() {
	make_aumid_rs();
    tauri_build::build()
}
