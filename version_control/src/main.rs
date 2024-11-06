use clap::{App, Arg, SubCommand};
use std::process;

mod commands;
mod repository;
mod utils;

fn main() {
    let matches = App::new("mini-git")
        .version("1.0")
        .author("Your Name")
        .about("A simple version control system")
        .subcommand(
            SubCommand::with_name("init")
                .about("Initialize a new repository"),
        )
        .subcommand(
            SubCommand::with_name("add")
                .about("Add files to staging area")
                .arg(
                    Arg::with_name("paths")
                        .help("Paths to add")
                        .required(true)
                        .multiple(true)
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("commit")
                .about("Commit changes")
                .arg(
                    Arg::with_name("message")
                        .short("m")
                        .long("message")
                        .help("Commit message")
                        .required(true)
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("history")
                .about("Show commit history"),
        )
        .subcommand(
            SubCommand::with_name("push")
                .about("Push changes to remote"),
        )
        .subcommand(
            SubCommand::with_name("pull")
                .about("Pull changes from remote"),
        )
        .subcommand(
            SubCommand::with_name("checkout")
                .about("Checkout a specific commit")
                .arg(
                    Arg::with_name("commit_id")
                        .help("Commit ID to checkout")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("loadlast")
                .about("Checkout the most recent commit"),
        )
        .subcommand(
            SubCommand::with_name("diff")
                .about("Show changes between commits or working directory")
                .arg(
                    Arg::with_name("commit_id1")
                        .help("First commit ID (optional)")
                        .index(1),
                )
                .arg(
                    Arg::with_name("commit_id2")
                        .help("Second commit ID (optional)")
                        .index(2),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        ("init", Some(_)) => {
            if let Err(e) = commands::init() {
                eprintln!("Error initializing repository: {}", e);
                process::exit(1);
            }
        }
        ("add", Some(add_matches)) => {
            let paths: Vec<String> = add_matches
                .values_of("paths")
                .unwrap()
                .map(String::from)
                .collect();
            
            if let Err(e) = commands::add(&paths) {
                eprintln!("Error adding files: {}", e);
                process::exit(1);
            }
        }
        ("commit", Some(commit_matches)) => {
            let message = commit_matches.value_of("message").unwrap();
            if let Err(e) = commands::commit(message) {
                eprintln!("Error committing changes: {}", e);
                process::exit(1);
            }
        }
        ("history", Some(_)) => {
            if let Err(e) = commands::history() {
                eprintln!("Error showing history: {}", e);
                process::exit(1);
            }
        }
        ("push", Some(_)) => {
            if let Err(e) = commands::push() {
                eprintln!("Error pushing changes: {}", e);
                process::exit(1);
            }
        }
        ("pull", Some(_)) => {
            if let Err(e) = commands::pull() {
                eprintln!("Error pulling changes: {}", e);
                process::exit(1);
            }
        }
        ("checkout", Some(checkout_matches)) => {
            let commit_id = checkout_matches.value_of("commit_id").unwrap();
            if let Err(e) = commands::checkout(commit_id) {
                eprintln!("Error checking out commit: {}", e);
                process::exit(1);
            }
        }
        ("loadlast", Some(_)) => {
            if let Err(e) = commands::loadlast() {
                eprintln!("Error loading last commit: {}", e);
                process::exit(1);
            }
        }
        ("diff", Some(diff_matches)) => {
            let commit_id1 = diff_matches.value_of("commit_id1");
            let commit_id2 = diff_matches.value_of("commit_id2");
            if let Err(e) = commands::diff(commit_id1, commit_id2) {
                eprintln!("Error showing diff: {}", e);
                process::exit(1);
            }
        }
        _ => {
            println!("No command specified. Use --help for usage information.");
            process::exit(1);
        }
    }
}
