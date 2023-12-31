use crate::{instalation_options, instalation_struct};

use std::path::PathBuf;

use semver::Version;
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(untagged)]
/// A source uuid either a uuid or null for auto generated uuid
pub enum SourceUUID {
    Uuid(Uuid),
    #[serde(rename = "<UUID>")]
    Null,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
/// A source download
pub enum SourceDownload {
    /// We don't have the right to distribute the file
    ExternalResource(Url),
    /// We have the right to distribute the file
    LocalResource(PathBuf),
}

instalation_struct! {
    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
    #[serde(rename_all = "camelCase")]
    /// Cabextract instalation type
    pub struct CabextractInstalationSource, CabextractInstalationCompiled {
        /// The cabextract file
        pub files: Vec<String>,
    }
}

instalation_options! {
    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
    #[serde(rename_all = "camelCase")]
    #[serde(tag = "type")]
    /// Installation type
    pub enum {
        Cabextract(CabextractInstalationSource, CabextractInstalationCompiled)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
/// The font category
pub enum FontCategory {
    Serif = 4,
    SansSerif = 3,
    Monospace = 2,
    Cursive = 0,
    Display = 1,
    Symbol = 5,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
/// A group of fonts
pub struct SourceGroup {
    pub id: SourceUUID,
    pub name: String,
    pub fonts: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
/// A group of fonts
pub struct CompiledGroup {
    pub id: Uuid,
    pub name: String,
    pub fonts: Vec<Uuid>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
/// A font
pub struct SourceFont {
    pub id: SourceUUID,
    pub name: String,
    pub short_name: String,
    pub publisher: String,
    pub categories: Vec<FontCategory>,
    pub installations: Vec<SourceInstalationType>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
/// A font
pub struct CompiledFont {
    pub id: Uuid,
    pub name: String,
    pub short_name: String,
    pub publisher: String,
    pub categories: Vec<FontCategory>,
    pub installations: Vec<CompiledInstalationType>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
/// Downloads
pub struct CompiledDownloads {
    pub id: Uuid,
    pub file_size: u64,
    pub hash: String,
    pub download_url: Url,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// The file format (of the source)
pub struct Source {
    pub groups: Vec<SourceGroup>,
    pub fonts: Vec<SourceFont>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// The file format (of the compiled)
pub struct Compiled {
    pub version: Version,
    pub downloads: Vec<CompiledDownloads>,
    pub groups: Vec<CompiledGroup>,
    pub fonts: Vec<CompiledFont>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// The references to the versions
pub struct FileFormatReference {
    pub id: Uuid,
    pub version: Version,
    pub download_url: Url,
}

pub type FileFormatReferences = Vec<FileFormatReference>;
