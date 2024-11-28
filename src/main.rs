use anyhow::Result;
use clap::{Parser, Subcommand};
use github_db::{Document, GithubDb};
use serde_json::Value;
use std::{path::PathBuf, fs, io::{self, Read}, env};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the database directory
    #[arg(short, long, default_value = ".github-db", env = "DB_PATH")]
    path: PathBuf,

    /// Encryption key (optional)
    #[arg(short, long, env = "DB_KEY")]
    key: Option<String>,

    /// Certificate file for authentication
    #[arg(short, long, env = "DB_CERT")]
    cert: Option<PathBuf>,

    /// Certificate content (base64 encoded)
    #[arg(long, env = "DB_CERT_CONTENT")]
    cert_content: Option<String>,

    /// Read data from stdin instead of command line
    #[arg(long)]
    stdin: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new document
    Create {
        /// Document ID
        id: String,
        /// JSON data (optional if --stdin is used)
        data: Option<String>,
    },
    /// Read a document
    Read {
        /// Document ID
        id: String,
    },
    /// Update a document
    Update {
        /// Document ID
        id: String,
        /// JSON data (optional if --stdin is used)
        data: Option<String>,
    },
    /// Delete a document
    Delete {
        /// Document ID
        id: String,
    },
    /// List all documents
    List,
    /// Generate a new certificate
    GenerateCert {
        /// Username
        username: String,
        /// Output directory for certificate and key
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Revoke a certificate
    RevokeCert {
        /// Username
        username: String,
    },
    /// List all valid certificates
    ListCerts,
}

fn read_stdin() -> Result<String> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    Ok(buffer)
}

fn get_json_data(data_arg: Option<String>, stdin: bool) -> Result<Value> {
    let data_str = if stdin {
        read_stdin()?
    } else {
        data_arg.ok_or_else(|| anyhow::anyhow!("No data provided. Use --stdin or provide data argument"))?
    };
    Ok(serde_json::from_str(&data_str)?)
}

fn print_document(doc: &Document) {
    if env::var("DB_JSON_OUTPUT").is_ok() {
        println!("{}", serde_json::to_string(doc).unwrap());
    } else {
        println!("ID: {}", doc.id);
        println!("Created: {}", doc.created_at);
        println!("Updated: {}", doc.updated_at);
        println!("Data: {}", serde_json::to_string_pretty(&doc.data).unwrap());
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Get certificate from file or content
    let cert_data = match (cli.cert, cli.cert_content) {
        (Some(path), _) => fs::read(path)?,
        (_, Some(content)) => base64::decode(content)?,
        _ => anyhow::bail!("Certificate required. Provide --cert or --cert-content"),
    };

    let encryption_key = cli.key.as_deref().map(str::as_bytes);
    let mut db = GithubDb::new(cli.path, encryption_key)?;

    // Handle certificate-based commands separately
    match &cli.command {
        Some(Commands::GenerateCert { username, output }) => {
            let (cert, key) = db.generate_certificate(username)?;
            fs::create_dir_all(output)?;
            fs::write(output.join(format!("{}.cert", username)), cert)?;
            fs::write(output.join(format!("{}.key", username)), key)?;
            println!("Certificate generated for {}", username);
            println!("Files saved in: {}", output.display());
            return Ok(());
        }
        Some(Commands::RevokeCert { username }) => {
            db.revoke_certificate(username)?;
            println!("Certificate revoked for {}", username);
            return Ok(());
        }
        Some(Commands::ListCerts) => {
            let certs = db.list_certificates()?;
            println!("Valid certificates:");
            for cert in certs {
                println!("- {}", cert);
            }
            return Ok(());
        }
        None => {
            anyhow::bail!("No command provided");
        }
        _ => {}
    }

    // Verify certificate for data operations
    if !db.verify_certificate(&cert_data)? {
        anyhow::bail!("Invalid or revoked certificate");
    }

    match cli.command.unwrap() {
        Commands::Create { id, data } => {
            let value = get_json_data(data, cli.stdin)?;
            let doc = db.create(&id, value)?;
            print_document(&doc);
        }
        Commands::Read { id } => {
            let doc = db.read(&id)?;
            print_document(&doc);
        }
        Commands::Update { id, data } => {
            let value = get_json_data(data, cli.stdin)?;
            let doc = db.update(&id, value)?;
            print_document(&doc);
        }
        Commands::Delete { id } => {
            db.delete(&id)?;
            println!("Document {} deleted successfully", id);
        }
        Commands::List => {
            let docs = db.list()?;
            if env::var("DB_JSON_OUTPUT").is_ok() {
                println!("{}", serde_json::to_string(&docs)?);
            } else {
                println!("Documents:");
                for id in docs {
                    println!("- {}", id);
                }
            }
        }
        _ => unreachable!(),
    }

    Ok(())
}
