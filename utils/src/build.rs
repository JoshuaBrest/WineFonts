use std::{collections::HashMap, fmt::Display, path::PathBuf};

use semver::Version;
use url::Url;
use uuid::Uuid;

use crate::{
    types::{
        CabextractInstalationCompiled, Compiled, CompiledDownloads, CompiledInstalationType,
        Source, SourceDownload, SourceInstalationType, SourceUUID,
    },
    utils::{generate_url, DownloadsList, UploadableDownloadInfo},
};

pub enum BuildError {
    /// Unexpected empty uuid (name)
    UnexpectedEmptyUuid(String),

    /// A font is missing (name)
    MissingFont(String),

    /// The download failed (URL, error)
    DownloadFailed(Url, String),

    /// File not found (path, error)
    FileError(PathBuf, String),
}

impl Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildError::UnexpectedEmptyUuid(name) => {
                write!(f, "Unexpected empty uuid (name: {})", name)
            }
            BuildError::MissingFont(name) => write!(f, "Missing font (name: {})", name),
            BuildError::DownloadFailed(url, error) => {
                write!(f, "Download failed (url: {}, error: {})", url, error)
            }
            BuildError::FileError(path, error) => {
                write!(f, "File error (path: {}, error: {})", path.display(), error)
            }
        }
    }
}

pub async fn build(
    version: Version,
    source: &Source,
    base_url: Url,
    base_path: PathBuf,
    downloadables: DownloadsList,
) -> Result<(Vec<UploadableDownloadInfo>, Compiled), BuildError> {
    let mut built = Compiled {
        version,
        groups: vec![],
        fonts: vec![],
        downloads: vec![],
    };

    let mut new_downloads: Vec<UploadableDownloadInfo> = vec![];

    let mut check_download: HashMap<SourceDownload, Vec<Uuid>> = HashMap::new();

    for group in &source.groups {
        let mut fonts: Vec<Uuid> = vec![];

        // Loop through the fonts
        for font_item in &group.fonts {
            let font = source.fonts.iter().find(|f| &f.name == font_item);

            // Check if the font exists
            if let Some(uuid) = font {
                fonts.push(match uuid.id {
                    SourceUUID::Uuid(uuid) => uuid,
                    SourceUUID::Null => {
                        return Err(BuildError::UnexpectedEmptyUuid(format!(
                            "group -> font: {}",
                            font_item
                        )))
                    }
                });
            } else {
                return Err(BuildError::MissingFont(font_item.clone()));
            }
        }

        built.groups.push(crate::types::CompiledGroup {
            id: match group.id {
                SourceUUID::Uuid(uuid) => uuid,
                SourceUUID::Null => {
                    return Err(BuildError::UnexpectedEmptyUuid(format!(
                        "group: {}",
                        group.name
                    )))
                }
            },
            name: group.name.clone(),
            fonts,
        });
    }

    // Add the fonts
    for font in &source.fonts {
        let id = match font.id {
            SourceUUID::Uuid(uuid) => uuid,
            SourceUUID::Null => {
                return Err(BuildError::UnexpectedEmptyUuid(format!(
                    "font: {}",
                    font.name
                )))
            }
        };

        let mut installations: Vec<CompiledInstalationType> = vec![];

        for installation in &font.installations {
            // Insert a temp random uuid for the download
            let download_uuid = Uuid::new_v4();

            let download = match installation {
                SourceInstalationType::Cabextract(data) => &data.download,
            };

            // Push the download
            if let Some(downloads) = check_download.get_mut(&download) {
                downloads.push(download_uuid);
            } else {
                check_download.insert(download.clone(), vec![download_uuid]);
            }

            // Push the installation
            installations.push(match installation {
                SourceInstalationType::Cabextract(data) => {
                    CompiledInstalationType::Cabextract(CabextractInstalationCompiled {
                        download: download_uuid,
                        files: data.files.clone(),
                    })
                }
            });
        }

        built.fonts.push(crate::types::CompiledFont {
            id,
            name: font.name.clone(),
            short_name: font.short_name.clone(),
            publisher: font.publisher.clone(),
            categories: font.categories.clone(),
            installations,
        });
    }

    // Add the downloads
    for (download, uuids) in check_download {
        let bytes = match download {
            SourceDownload::ExternalResource(ref url) => match reqwest::get(url.clone()).await {
                Ok(data) => {
                    if data.status() != 200 {
                        return Err(BuildError::DownloadFailed(
                            url.clone(),
                            format!("Status code: {}", data.status()),
                        ));
                    }

                    match data.bytes().await {
                        Ok(data) => data.as_ref().to_vec(),
                        Err(e) => {
                            return Err(BuildError::DownloadFailed(url.clone(), e.to_string()))
                        }
                    }
                }
                Err(e) => return Err(BuildError::DownloadFailed(url.clone(), e.to_string())),
            },
            SourceDownload::LocalResource(ref path) => {
                let joined = base_path.join(&path);

                let data = match std::fs::read(joined) {
                    Ok(data) => data,
                    Err(e) => return Err(BuildError::FileError(path.clone(), e.to_string())),
                };

                data.as_slice().to_vec()
            }
        };

        let hash = sha256::digest(&bytes);
        let size = bytes.len() as u64;

        // Check if the download already exists
        let existing = downloadables.iter().find(|d| d.hash == hash);

        let id = match existing {
            Some(existing) => {
                built.downloads.push(CompiledDownloads {
                    id: existing.id,
                    file_size: size,
                    hash,
                    download_url: existing.download_url.clone(),
                });

                existing.id
            }
            None => {
                let id = Uuid::new_v4();

                let url = match download {
                    SourceDownload::ExternalResource(ref url) => url.clone(),
                    SourceDownload::LocalResource(ref path) => generate_url(&base_url, &path, &id),
                };

                built.downloads.push(CompiledDownloads {
                    id,
                    file_size: size,
                    hash: hash.clone(),
                    download_url: url.clone(),
                });

                if let SourceDownload::LocalResource(ref path) = download {
                    new_downloads.push(UploadableDownloadInfo {
                        uuid: id,
                        file_path: path.to_owned(),
                        hash,
                        file_size: size,
                    });
                }

                id
            }
        };

        // Replace the uuids
        for uuid in uuids {
            for font in &mut built.fonts {
                for installation in &mut font.installations {
                    match installation {
                        CompiledInstalationType::Cabextract(data) => {
                            if data.download == uuid {
                                data.download = id;
                            }
                        }
                    }
                }
            }
        }
    }

    Ok((new_downloads, built))
}
