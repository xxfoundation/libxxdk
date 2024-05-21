fn main() {
    if let Err(err) = dm_demo::run() {
        eprintln!("Error: {err}");
        std::process::exit(-1);
    }
}
