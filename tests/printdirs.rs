use std::env;
use xdgdir::BaseDir;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: print_dirs <app_name>");
        std::process::exit(1);
    }
    let app_name = &args[1];

    match BaseDir::new(app_name) {
        Ok(dirs) => {
            // Print each path in a simple, parseable key=value format.
            println!("home={}", dirs.home.display());
            println!("config={}", dirs.config.display());
            println!("data={}", dirs.data.display());
            println!("state={}", dirs.state.display());
            println!("cache={}", dirs.cache.display());
            println!("bin={}", dirs.bin.display());
            if let Some(runtime) = dirs.runtime {
                println!("runtime={}", runtime.display());
            }
        }
        Err(e) => {
            // If it fails, print the error to stderr and exit.
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
