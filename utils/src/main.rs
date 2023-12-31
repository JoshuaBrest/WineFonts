#[macro_use]
extern crate log;

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use s3::{creds::Credentials, Bucket, Region};
use semver::Version;
use serde::Serialize;
use tokio::fs;
use url::Url;

use crate::utils::{
    generate_versions_url, upload_version_to_s3, upload_versions_to_s3, VersionInfo,
};

pub mod build;
pub mod lint;
pub mod types;
pub mod utils;

#[macro_export]
macro_rules! instalation_struct {
    {
        $(#[$source_attr:meta])*
        $vis:vis struct $source_struct_name:ident, $compiled_struct_name:ident {
            $(
                $(#[$variant_attr:meta])*
                $attr_vis:vis $variant_name:ident: $variant_struct_name:ty,
            )*
        }
    } => {
        $(#[$source_attr])*
        $vis struct $source_struct_name {
            $vis download: SourceDownload,
            $(
                $(#[$variant_attr])*
                $attr_vis $variant_name: $variant_struct_name,
            )*
        }

        $(#[$source_attr])*
        $vis struct $compiled_struct_name {
            $vis download: Uuid,
            $(
                $(#[$variant_attr])*
                $attr_vis $variant_name: $variant_struct_name,
            )*
        }
    }
}

#[macro_export]
macro_rules! instalation_options {
    {
        $(#[$source_attr:meta])*
        $vis:vis enum {
            $(
                $(#[$variant_attr:meta])*
                $variant_name:ident($source_struct_name:ident, $compiled_struct_name:ident)
            )*
        }
    } => {
        $(#[$source_attr])*
        $vis enum SourceInstalationType {
            $(
                $(#[$variant_attr])*
                $variant_name($source_struct_name),
            )*
        }

        $(#[$source_attr])*
        $vis enum CompiledInstalationType {
            $(
                $(#[$variant_attr])*
                $variant_name($compiled_struct_name),
            )*
        }
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Lints the fonts.json file and updates the database
    Lint {
        #[clap(short, long)]
        /// Path to config (fonts.json)
        config: PathBuf,

        #[clap(long)]
        /// Path to the base directory
        base_path: PathBuf,

        #[clap(long)]
        /// Whether to fix the issues
        fix: bool,
    },
    /// Updates the database
    Update {
        #[clap(short, long)]
        /// Path to config (fonts.json)
        config: PathBuf,

        #[clap(long)]
        /// Version to insert
        version: Version,

        #[clap(long)]
        /// Base path
        base_path: PathBuf,

        #[clap(long, env)]
        /// Base access S3 url
        base_url: Url,

        /// S3 endpoint
        #[clap(long, env)]
        endpoint: String,

        /// S3 access key id
        #[clap(long, env)]
        access_key_id: String,

        /// S3 secret access key
        #[clap(long, env)]
        secret_access_key: String,

        /// S3 bucket
        #[clap(long, env)]
        bucket: String,
    },
}

#[derive(Parser)]
#[command(
    author = "WineFonts Team",
    about = "Lints the fonts.json file and updates the database"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

async fn file_from_path(path: PathBuf) -> Option<types::Source> {
    // Check valid file
    let file_contents = match fs::read_to_string(path).await {
        Ok(file) => file,
        Err(error) => {
            error!("Failed to read file: {}", error);
            return None;
        }
    };

    // Parse the file
    let json = match serde_json::from_str::<types::Source>(&file_contents) {
        Ok(json) => json,
        Err(error) => {
            error!("Failed to parse file: {}", error);
            return None;
        }
    };

    Some(json)
}

#[tokio::main]
async fn main() {
    // Dotenv
    dotenv::dotenv().ok();

    // Check if the log level is set in the env (if not, default to info)
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    // Init the logger
    pretty_env_logger::init();
    // CLI parser
    let parser = Cli::parse();

    match parser.command {
        Commands::Lint {
            config,
            base_path,
            fix,
        } => {
            let json = match file_from_path(config.clone()).await {
                Some(json) => json,
                None => return,
            };

            // If errors are found, print them and exit
            let (new_json, errors) = lint::lint(
                &json,
                base_path,
                match fix {
                    true => lint::LintMode::Fix,
                    false => lint::LintMode::Check,
                },
            )
            .await;
            if errors.len() > 0 {
                for error in &errors {
                    error!("{}", error);
                }
            }

            if errors.len() > 0 {
                warn!("Found {} unresolved errors", errors.len());
            } else {
                info!("No errors found");
            }

            // Write the new json
            if fix {
                let mut buf = Vec::new();
                let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
                let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
                match new_json.serialize(&mut ser) {
                    Ok(_) => info!("Serialized json"),
                    Err(error) => {
                        error!("Failed to serialize json: {}", error);
                        std::process::exit(1);
                    }
                }

                let new_json_string = match String::from_utf8(buf) {
                    Ok(string) => string,
                    Err(error) => {
                        error!("Failed to convert json to string: {}", error);
                        std::process::exit(1);
                    }
                };

                // Write the new json
                match fs::write(config, new_json_string).await {
                    Ok(_) => info!("Wrote new json"),
                    Err(error) => {
                        error!("Failed to write new json: {}", error);
                        std::process::exit(1);
                    }
                }
            }
        }
        Commands::Update {
            config,
            base_path,
            endpoint,
            access_key_id,
            secret_access_key,
            bucket,
            base_url,
            version,
        } => {
            // Get the json
            let json = match file_from_path(config.clone()).await {
                Some(json) => json,
                None => return,
            };

            // Check for any lint errors
            let (_, errors) = lint::lint(&json, base_path.clone(), lint::LintMode::Check).await;
            if errors.len() > 0 {
                for error in &errors {
                    error!("{}", error);
                }
                warn!("Found {} unresolved errors", errors.len());
                error!("Please fix any unresolved errors before updating the database");
                std::process::exit(1);
            }

            info!("No errors found");

            let region = Region::Custom {
                region: "us-east-1".to_string(),
                endpoint: endpoint.to_string(),
            };

            let creds = match Credentials::new(
                Some(&access_key_id),
                Some(&secret_access_key),
                None,
                None,
                None,
            ) {
                Ok(creds) => creds,
                Err(error) => {
                    error!("Failed to create credentials: {}", error);
                    std::process::exit(1);
                }
            };

            // Get the s3 client
            let s3 = match Bucket::new(&bucket, region, creds) {
                Ok(s3) => s3,
                Err(error) => {
                    error!("Failed to create s3 client: {}", error);
                    std::process::exit(1);
                }
            };

            // Get the downloadables
            let downloadables = utils::grab_downloadables_from_s3(&s3).await;

            // Build the database
            let (new, file) = match build::build(
                version.clone(),
                &json,
                base_url.clone(),
                base_path.clone(),
                downloadables.clone(),
            )
            .await
            {
                Ok(built) => built,
                Err(error) => {
                    error!("Failed to build database: {}", error);
                    std::process::exit(1);
                }
            };

            // Upload the database
            utils::upload_files_to_s3(&s3, &base_url, base_path, downloadables, new).await;

            // New UUID
            let new_uuid = uuid::Uuid::new_v4();

            // Serialize the file
            let file = match serde_json::to_vec(&file) {
                Ok(file) => file,
                Err(error) => {
                    error!("Failed to serialize file: {}", error);
                    std::process::exit(1);
                }
            };

            // Upload the file
            upload_version_to_s3(&s3, new_uuid, &file).await;

            // Get version list
            let mut versions = utils::grab_versions_from_s3(&s3).await;

            // Add the new version
            versions.push(VersionInfo {
                id: new_uuid,
                version,
                download_url: generate_versions_url(&base_url, &new_uuid),
                hash: sha256::digest(&file),
                file_size: file.len() as u64,
            });

            // Upload the versions
            upload_versions_to_s3(&s3, versions).await;
        }
    }
}
