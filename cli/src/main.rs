mod commands;
mod config;
mod output;
mod server;

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = if args.first().is_some_and(|command| command == "config") {
        commands::run_config_command(&args)
    } else {
        commands::run_cli(&args)
    };

    match result {
        Ok(result) => {
            println!("{result}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{}", output::error_json(&error.code, &error.message));
            ExitCode::from(1)
        }
    }
}
