use std::process::ExitCode;

use nota::NotaEncode;
use skills::CommandLine;

fn main() -> ExitCode {
    match CommandLine::from_environment().run() {
        Ok(output) => {
            println!("{}", output.to_nota());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("skills: {error}");
            ExitCode::FAILURE
        }
    }
}
