use clap::{App, Arg};
use solc::{Compiler, Input};
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("tron-solc")
        .version("0.5.15")
        .author("andelf <andelf@gmail.com>")
        .about("Does awesome things")
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .required(true)
                .value_name("DIR")
                .about("Output directory"),
        )
        .arg(Arg::new("INPUT").required(true).about("Input Contract file"))
        .get_matches();

    let fname = matches.value_of("INPUT").unwrap();
    let code = fs::read_to_string(fname)?;

    let outdir = matches.value_of("output").unwrap();
    fs::create_dir_all(outdir)?;

    let input = Input::new().optimizer(0).source(fname, code.into());
    let output = Compiler::new().unwrap().compile(input).unwrap();

    if output.has_errors() {
        output.format_errors();
    }

    for (import_name, _cntr) in output.contracts.iter() {
        let fpath = Path::new(import_name);
        let fname = fpath.file_name().unwrap().to_str().unwrap();
        let name = fpath.file_stem().unwrap().to_str().unwrap();
        println!("I: {} => {}", fname, import_name);
        let mut output_path = Path::new(outdir).to_path_buf();
        output_path.push(name);

        fs::write(output_path.with_extension("abi"), output.pretty_abi_of(name)?)?;
        fs::write(output_path.with_extension("bin"), output.bytecode_of(name)?)?;
    }
    Ok(())
}
