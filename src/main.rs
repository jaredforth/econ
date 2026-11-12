fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    match thrift::user_env().and_then(|env| thrift::run(&args, env)) {
        Ok(out) => println!("{out}"),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    }
}
