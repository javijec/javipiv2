use std::process::ExitCode;

fn main() -> ExitCode {
    match javipiv2::interfaces::cli::run(std::env::args_os().skip(1)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::from(1)
        }
    }
}
