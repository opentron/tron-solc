use clap::{App, Arg};
use solc::{Compiler, Input};
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("tron-solc")
        .version("0.5.15")
        .author("andelf <andelf@gmail.com>")
        .about("The all-in-one tron-solidity compiler.")
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .required(true)
                .value_name("DIR")
                .about("Output directory"),
        )
        .arg(
            Arg::new("optimizer-runs")
                .long("optimizer-runs")
                .value_name("n")
                .default_value("0")
                .about("Runs of optimizer"),
        )
        .arg(
            Arg::new("proxy")
                .long("proxy")
                .value_name("URL")
                .env("ALL_PROXY")
                .about("Proxy server"),
        )
        .arg(
            Arg::new("INPUT")
                .required(true)
                .about("Input Contract file"),
        )
        .get_matches();

    let fname = matches.value_of("INPUT").unwrap();
    let code = fs::read_to_string(fname)?;

    let outdir = matches.value_of("output").unwrap();
    fs::create_dir_all(outdir)?;

    let optimizer_runs = matches
        .value_of("optimizer-runs")
        .expect("has default; qed")
        .parse()?;

    if let Some(url) = matches.value_of("proxy") {
        solc::set_proxy(url);
    }

    let input = Input::new()
        .optimizer(optimizer_runs)
        .source(fname, code.into());
    let output = Compiler::new()?.compile(input)?;

    if output.has_errors() {
        output.format_errors();
    }

    for (import_name, _cntr) in output.contracts.iter() {
        let fpath = Path::new(import_name);
        let fname = fpath.file_name().unwrap().to_str().unwrap();
        let name = fpath.file_stem().unwrap().to_str().unwrap();
        println!("I: Writing {} => {}", fname, import_name);
        let mut output_path = Path::new(outdir).to_path_buf();
        output_path.push(name);

        fs::write(
            output_path.with_extension("abi"),
            output.pretty_abi_of(name)?,
        )?;
        fs::write(output_path.with_extension("bin"), output.bytecode_of(name)?)?;
    }
    Ok(())
}
