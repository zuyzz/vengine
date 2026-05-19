use vengine::cli;

fn main() {
    let cli = cli::run();
    if let Err(e) = cli {
        eprintln!("{}", e);
    }
}
