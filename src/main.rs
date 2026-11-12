fn main() {
    match thrift::run(&std::env::args().skip(1).collect::<Vec<_>>(), |k| {
        std::env::var(k).ok()
    }) {
        Ok(out) => println!("{out}"),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    }
}
