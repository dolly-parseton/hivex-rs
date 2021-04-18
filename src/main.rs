use hivex_rs;

use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(
    name = "hivex-rs-cli",
    about = "Read one or more Windows Registry files and apply some simple formatting options."
)]
struct Opt {
    #[structopt(parse(from_os_str))]
    files: Vec<PathBuf>,
}

fn main() {
    let mut opt = Opt::from_args();
    //
    let start = std::time::Instant::now();
    for file in opt.files {
        let mut hive = hivex_rs::hive::Hive::new(file).unwrap();
        while let Some(node) = hive.next() {
            println!("{}", node.unwrap());
        }
    }

    println!("Ms: {}", start.elapsed().as_millis());
}
