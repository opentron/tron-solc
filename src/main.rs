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
            Arg::new("optimize")
                .long("optimize")
                .about("Enable bytecode optimizer"),
        )
        .arg(
            Arg::new("optimizer-runs")
                .long("optimizer-runs")
                .value_name("n")
                .default_value("0")
                .requires("optimize")
                .about("Runs of optimizer"),
        )
        .arg(
            Arg::new("proxy")
                .long("proxy")
                .value_name("URL")
                .env("ALL_PROXY")
                .about("Proxy server"),
        )
        .arg(Arg::new("layout").long("layout").about("Ouput storage layout"))
        .arg(Arg::new("INPUT").required(true).about("Input Contract file"))
        .get_matches();

    let fname = matches.value_of("INPUT").unwrap();
    let code = fs::read_to_string(fname)?;

    let outdir = matches.value_of("output").unwrap();
    fs::create_dir_all(outdir)?;

    if let Some(url) = matches.value_of("proxy") {
        solc::set_proxy(url);
    }

    let (enabled, runs) = if matches.is_present("optimize") {
        (true, matches.value_of("optimizer-runs").unwrap().parse()?)
    } else {
        (false, 0)
    };

    let input = Input::new()
        .optimizer(enabled, runs)
        .source(fname, code.into());
    let output = Compiler::new()?.compile(input)?;

    if output.has_errors() {
        output.format_errors();
    }

    for (file_name, cntrs) in output.contracts.iter() {
        for (cntr_name, cntr) in cntrs.iter() {
            let fpath = Path::new(cntr_name);
            let fname = fpath.file_name().unwrap().to_str().unwrap();
            let name = fpath.file_stem().unwrap().to_str().unwrap();
            println!("I: writing {}:{} to {}", file_name, fname, fname);
            let mut output_path = Path::new(outdir).to_path_buf();
            output_path.push(name);

            fs::write(output_path.with_extension("abi"), output.pretty_abi_of(name)?)?;
            fs::write(output_path.with_extension("bin"), output.bytecode_of(name)?)?;

            if matches.is_present("layout") && cntr.has_storage_layout() {
                println!("I: writing {}.layout", cntr_name);
                fs::write(
                    output_path.with_extension("layout"),
                    serde_json::to_string_pretty(&cntr.storage_layout)?,
                )?;
            }
        }
    }
    Ok(())
}
