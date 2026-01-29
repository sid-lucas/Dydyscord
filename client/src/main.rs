use clap::{Parser, Subcommand};
use reqwest;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Arguments globaux (pour toutes les commandes)
    #[arg(short, long)]
    verbose: bool,

    // Subcommands dispo (Commands)
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Connect {
        #[arg(short, long, default_value = "127.0.0.1")]
        ip: String,

        #[arg(short, long, default_value = "2727")]
        port: u16,
    }
}

fn main() {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Connect {ip, port } => {
            println!("Connecting to {}:{}...", ip, port);

            // Curl du client sur le serv :
            let url = format!("http://{}:{}/health", ip, port);
            match reqwest::blocking::get(&url) {
                Ok(response) => {
                    if response.status().is_success() {
                        println!("Connected successfully!");
                        println!("Serveur response: {}", response.text().unwrap());
                    }
                },
                Err(e) => { eprintln!("{}", e);},
            }
        }
    }
}