use econ::Out;
use std::io::{IsTerminal, Read};
use std::os::unix::process::CommandExt;

fn die(msg: String) -> ! {
    eprintln!("{msg}");
    std::process::exit(2)
}

fn main() {
    let mut args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() && !std::io::stdin().is_terminal() {
        // piped input can't drive an interactive session: just print the model
        let mut piped = String::new();
        let _ = std::io::stdin().read_to_string(&mut piped);
        args = vec!["-n".into(), piped];
    }
    let env = econ::user_env().unwrap_or_else(|e| die(e));
    match econ::plan(&args, &env) {
        Ok(Out::Print(s)) => println!("{s}"),
        Ok(Out::Launch { model, prompt }) => {
            let claude = env("ECON_CLAUDE").unwrap_or("claude".into());
            let e = std::process::Command::new(&claude)
                .args(["--model", &model, &prompt])
                .exec();
            die(format!("econ: can't run `{claude}`: {e}"))
        }
        Err(e) => die(e),
    }
}
