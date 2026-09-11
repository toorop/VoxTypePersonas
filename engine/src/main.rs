use std::process::ExitCode;

fn main() -> ExitCode {
    match voxtype_personas::cli::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("voxtype-personas: {error}");
            ExitCode::from(2)
        }
    }
}
