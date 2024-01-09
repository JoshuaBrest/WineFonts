use std::path::PathBuf;

use s3::Bucket;
use semver::Version;
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Downloadable {
    pub id: Uuid,
    pub file_size: u64,
    pub hash: String,
    pub download_url: Url,
}

pub type DownloadsList = Vec<Downloadable>;

pub async fn grab_downloadables_from_s3(s3: &Bucket) -> DownloadsList {
    match s3.get_object("/downloadables.json").await {
        Ok(data) => {
            let data = serde_json::from_slice(data.as_slice());

            match data {
                Ok(data) => data,
                Err(e) => {
                    // File is corrupted
                    error!("Failed to parse downloadables.json: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            // File probably doesn't exist
            warn!(
                "Failed to get downloadables.json: {}... Using empty list",
                e
            );
            vec![]
        }
    }
}

pub const DOWNLOAD_FILE_PATH: &str = "downloads";
pub const VERSIONS_FILE_PATH: &str = "versions";

pub fn generate_versions_url(base_url: &Url, id: &Uuid) -> Url {
    let mut url = base_url.clone();
    let mut url_path = base_url.path_segments().unwrap().collect::<Vec<_>>();

    let data = urlencoding::encode(&VERSIONS_FILE_PATH).into_owned();
    url_path.push(data.as_str());

    let data = format!("{}.json", id);
    let data = urlencoding::encode(&data).into_owned();
    url_path.push(data.as_str());

    url.set_path(&url_path.join("/"));

    url
}

pub async fn upload_version_to_s3(s3: &Bucket, id: Uuid, built: &Vec<u8>) {
    let mut path: PathBuf = [VERSIONS_FILE_PATH, &id.to_string()].iter().collect();
    path.set_extension("json");

    match s3.put_object_with_content_type(path.to_str().unwrap(), &built, "application/json").await {
        Ok(_) => info!("Uploaded version {}.json", id),
        Err(e) => {
            error!("Failed to upload version {}.json: {}", id, e);
            std::process::exit(1);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionInfo {
    pub id: Uuid,
    pub version: Version,
    pub download_url: Url,
    pub hash: String,
    pub file_size: u64,
}

pub async fn grab_versions_from_s3(s3: &Bucket) -> Vec<VersionInfo> {
    match s3.get_object("/versions.json").await {
        Ok(data) => {
            let data = serde_json::from_slice(data.as_slice());

            match data {
                Ok(data) => data,
                Err(e) => {
                    // File is corrupted
                    error!("Failed to parse versions.json: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            // File probably doesn't exist
            warn!("Failed to get versions.json: {}... Using empty list", e);
            vec![]
        }
    }
}

pub async fn upload_versions_to_s3(s3: &Bucket, versions: Vec<VersionInfo>) {
    let data = match serde_json::to_vec(&versions) {
        Ok(data) => data,
        Err(e) => {
            error!("Failed to serialize versions.json: {}", e);
            std::process::exit(1);
        }
    };

    match s3.put_object_with_content_type("/versions.json", &data, "application/json").await {
        Ok(_) => info!("Uploaded versions.json"),
        Err(e) => {
            error!("Failed to upload versions.json: {}", e);
            std::process::exit(1);
        }
    }
}

pub fn generate_url(base_url: &Url, path: &PathBuf, uuid: &Uuid) -> Url {
    let mut url = base_url.clone();
    let mut url_path = base_url.path_segments().unwrap().collect::<Vec<_>>();

    let file_extension = match path.extension() {
        Some(extension) => ".".to_string() + extension.to_str().unwrap(),
        None => "".to_string(),
    };

    let file_name = format!("{}{}", uuid.to_string(), file_extension);

    let data = urlencoding::encode(&DOWNLOAD_FILE_PATH).into_owned();
    url_path.push(data.as_str());

    let data = urlencoding::encode(&file_name).into_owned();
    url_path.push(data.as_str());

    url.set_path(&url_path.join("/"));

    url
}

pub struct UploadableDownloadInfo {
    pub uuid: Uuid,
    pub file_path: PathBuf,
    pub hash: String,
    pub file_size: u64,
}

pub async fn upload_files_to_s3(
    s3: &Bucket,
    base_url: &Url,
    base_path: PathBuf,
    original_downloads: DownloadsList,
    downloads: Vec<UploadableDownloadInfo>,
) {
    let mut new_downloads = original_downloads.clone();

    // Loop through the downloads
    for download in downloads {
        let UploadableDownloadInfo {
            uuid,
            file_path,
            hash,
            file_size,
        } = download;

        // Upload the file
        let data = match std::fs::read(base_path.join(&file_path)) {
            Ok(data) => data,
            Err(e) => {
                error!("Failed to read file: {}", e);
                std::process::exit(1);
            }
        };

        let mut path: PathBuf = [DOWNLOAD_FILE_PATH, &uuid.to_string()].iter().collect();

        path.set_extension(file_path.extension().unwrap());

        match s3.put_object(path.to_str().unwrap(), &data).await {
            Ok(_) => info!("Uploaded file: {}", path.to_str().unwrap()),
            Err(e) => {
                error!("Failed to upload file: {}", e);
                std::process::exit(1);
            }
        }

        // Add the download to the list
        new_downloads.push(Downloadable {
            id: uuid,
            file_size,
            hash,
            download_url: generate_url(&base_url, &file_path, &uuid),
        });
    }

    // Upload the downloadables.json file
    let data = match serde_json::to_vec(&new_downloads) {
        Ok(data) => data,
        Err(e) => {
            error!("Failed to serialize downloadables.json: {}", e);
            std::process::exit(1);
        }
    };

    match s3.put_object_with_content_type("downloadables.json", &data, "application/json").await {
        Ok(_) => info!("Uploaded downloadables.json"),
        Err(e) => {
            error!("Failed to upload downloadables.json: {}", e);
            std::process::exit(1);
        }
    }
}
