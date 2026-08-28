mod cli;

fn main() -> std::process::ExitCode {
    match cli::run() {
        Ok(status) => status,
        Err(error) => {
            eprintln!("alofmt: {error:#}");
            std::process::ExitCode::from(2)
        }
    }
}
