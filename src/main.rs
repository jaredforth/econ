use std::io::{IsTerminal, Read};

fn main() {
    let mut args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() && !std::io::stdin().is_terminal() {
        let mut piped = String::new();
        let _ = std::io::stdin().read_to_string(&mut piped);
        args.push(piped);
    }
    match econ::user_env().and_then(|env| econ::run(&args, env)) {
        Ok(out) => println!("{out}"),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    }
}
