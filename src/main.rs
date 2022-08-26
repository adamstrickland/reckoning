use clap::Parser;
extern crate exitcode;
use std::process;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli{
    #[clap(value_parser)]
    file: String,
}


fn main() {
    let cli = Cli::parse();

    if let Err(err) = reckoning::run(cli.file.clone()) {
        eprintln!("Error reading from file at {}: {}", cli.file, err);
        process::exit(exitcode::USAGE);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        crate::Cli::command().debug_assert();
    }
}

