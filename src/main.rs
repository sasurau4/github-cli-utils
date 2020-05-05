use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    pattern: String,
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

fn main() -> io::Result<()> {
    let args = Cli::from_args();
    // let content = std::fs::read_to_string(&args.path).expect("could not read file");
    let f = File::open(args.path).expect("no file");
    let reader = BufReader::new(f);

    for line in reader.lines() {
        let copied_line = line?.clone();
        if copied_line.contains(&args.pattern) {
            println!("{}", copied_line);
        }
    }
    Ok(())
}
