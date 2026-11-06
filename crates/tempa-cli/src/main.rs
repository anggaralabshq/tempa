use clap::Parser;

#[derive(Parser)]
#[command(name = "tempa", version, about = "Satu kontrak konteks untuk semua coding agent")]
struct Cli {}

fn main() {
    let _cli = Cli::parse();
}
