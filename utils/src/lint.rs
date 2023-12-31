use std::{collections::HashMap, fmt::Display, path::PathBuf};

use url::Url;
use uuid::Uuid;

use crate::types::{Source, SourceDownload, SourceInstalationType, SourceUUID};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum LintMode {
    /// Check the source for errors
    Check,
    /// Fix the source
    Fix,
}

#[derive(PartialEq, Eq, Clone)]
pub enum ErrorContext {
    /// -> Group
    Groups,
    /// -> Font
    Fonts,
    /// -> Group -> Name
    Group(String),
    /// -> Font -> Name
    Font(String),
}

impl Display for ErrorContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorContext::Groups => write!(f, "groups"),
            ErrorContext::Fonts => write!(f, "fonts"),
            ErrorContext::Group(name) => write!(f, "groups -> {}", name),
            ErrorContext::Font(name) => write!(f, "fonts -> {}", name),
        }
    }
}

pub enum LintErrors {
    /* Common */
    /// The UUID has been reused (UUID)
    ReusedUuid(Uuid),
    /// Missing UUID (Name, Context)
    MissingUuid(ErrorContext),
    /// The name has been reused (Name, Context)
    DuplicatedName(String, ErrorContext),
    /// The name is too long (Name, Context)
    NameTooLong(String, ErrorContext),
    /// The name is too short (Name, Context)
    NameTooShort(String, ErrorContext),
    /// Unsorted list (Context)
    UnsortedList(ErrorContext),

    /* Groups */
    /// The group has no fonts (Group name)
    GroupEmpty(ErrorContext),
    /// The group has a duplicate font (Group name, Font name)
    GroupDuplicateFont(ErrorContext, String),
    /// The group has a font that doesn't exist (Group name, Font name)
    GroupFontDoesntExist(ErrorContext, String),

    /* Fonts */
    /// The font has no installations (Context)
    FontEmpty(ErrorContext),

    /* Downloads */
    /// The local resource doesn't exist (Context, Path)
    DownloadLocalResourceDoesntExist(ErrorContext, PathBuf),
    /// The external resource isn't https (Context, Url)
    DownloadExternalResourceNotHttps(ErrorContext, Url),
    /// The external resource doesn't exist (Context, Url, Status)
    DownloadExternalResourceError(ErrorContext, Url, u16),
}

impl Display for LintErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            /* Common */
            LintErrors::ReusedUuid(uuid) => write!(f, "The UUID {} has been reused", uuid),
            LintErrors::MissingUuid(context) => write!(f, "Missing UUID for {}", context),
            LintErrors::DuplicatedName(name, context) => write!(
                f,
                "The name/short name {} has been reused in {}",
                name, context
            ),
            LintErrors::NameTooLong(name, context) => {
                write!(f, "The name/short name {} is too long in {}", name, context)
            }
            LintErrors::NameTooShort(name, context) => write!(
                f,
                "The name/short name {} is too short in {}",
                name, context
            ),
            LintErrors::UnsortedList(context) => write!(
                f,
                "The list \"{}\" is not sorted",
                match context {
                    ErrorContext::Groups => "groups".to_string(),
                    ErrorContext::Fonts => "fonts".to_string(),
                    ErrorContext::Group(name) => format!("groups -> {} -> fonts", name),
                    ErrorContext::Font(name) => format!("fonts -> {} -> Installations", name),
                }
            ),

            /* Groups */
            LintErrors::GroupEmpty(context) => write!(f, "The group \"{}\" has no fonts", context),
            LintErrors::GroupDuplicateFont(context, font) => write!(
                f,
                "The group \"{}\" has a duplicate font named \"{}\"",
                context, font
            ),
            LintErrors::GroupFontDoesntExist(context, font) => write!(
                f,
                "The group \"{}\" has a font named \"{}\" that doesn't exist",
                context, font
            ),

            /* Fonts */
            LintErrors::FontEmpty(context) => {
                write!(f, "There are no installations for the font \"{}\"", context)
            }

            /* Downloads */
            LintErrors::DownloadLocalResourceDoesntExist(context, path) => write!(
                f,
                "The local resource for \"{}\" doesn't exist at \"{}\"",
                context,
                path.display()
            ),
            LintErrors::DownloadExternalResourceNotHttps(context, url) => write!(
                f,
                "External resource for \"{}\" isn't https at \"{}\"",
                context, url
            ),
            LintErrors::DownloadExternalResourceError(context, url, status) => write!(
                f,
                "Failed to download the external resource for \"{}\" at \"{}\" with status code {}",
                context, url, status
            ),
        }
    }
}

const MAX_NAME_LENGTH: usize = 50;
const MIN_NAME_LENGTH: usize = 3;

fn check_name(name: &str, context: ErrorContext) -> Result<(), LintErrors> {
    if name.len() > MAX_NAME_LENGTH {
        Err(LintErrors::NameTooLong(name.to_string(), context))
    } else if name.len() < MIN_NAME_LENGTH {
        Err(LintErrors::NameTooShort(name.to_string(), context))
    } else {
        Ok(())
    }
}

fn check_sorted<T>(list: &Vec<T>, sort_fn: &dyn Fn(&T, &T) -> bool) -> bool {
    let mut sorted = true;
    for i in 0..list.len() - 1 {
        if !sort_fn(&list[i], &list[i + 1]) {
            sorted = false;
            break;
        }
    }

    sorted
}

fn check_or_create_uuid(
    uuid_map: &mut HashMap<Uuid, ()>,
    uuid: SourceUUID,
    context: ErrorContext,
    lint_mode: LintMode,
) -> Result<Uuid, LintErrors> {
    match uuid {
        SourceUUID::Uuid(id) => {
            if uuid_map.contains_key(&id) {
                Err(LintErrors::ReusedUuid(id))
            } else {
                uuid_map.insert(id, ());
                Ok(id)
            }
        }
        SourceUUID::Null => {
            if lint_mode == LintMode::Fix {
                let new_id = Uuid::new_v4();
                uuid_map.insert(new_id, ());
                Ok(new_id)
            } else {
                Err(LintErrors::MissingUuid(context))
            }
        }
    }
}

pub async fn lint(
    original: &Source,
    base_path: PathBuf,
    lint_mode: LintMode,
) -> (Source, Vec<LintErrors>) {
    let mut new = original.clone().to_owned();

    let mut errors = Vec::<LintErrors>::new();

    let mut uuids = HashMap::<Uuid, ()>::new();

    // Check groups are in the correct order
    if lint_mode == LintMode::Fix {
        new.groups.sort_by(|a, b| a.name.cmp(&b.name));
    } else {
        if !check_sorted(&new.groups, &|a, b| a.name < b.name) {
            errors.push(LintErrors::UnsortedList(ErrorContext::Groups));
        }
    }

    // Check any groups with duplicate names or no fonts or invalid ids
    let mut group_names = HashMap::<String, ()>::new();
    for group in &mut new.groups {
        // Check if the group name is valid
        if group_names.contains_key(&group.name) {
            errors.push(LintErrors::DuplicatedName(
                group.name.clone(),
                ErrorContext::Group(group.name.to_string()),
            ));
        }

        if let Err(error) = check_name(&group.name, ErrorContext::Group(group.name.to_string())) {
            errors.push(error);
        }

        // Check if the group has any fonts
        if group.fonts.len() == 0 {
            errors.push(LintErrors::GroupEmpty(ErrorContext::Group(
                group.name.to_string(),
            )));
        }

        // Check Uuid
        match check_or_create_uuid(
            &mut uuids,
            group.id,
            ErrorContext::Group(group.name.to_string()),
            lint_mode,
        ) {
            Ok(id) => {
                // If in fix mode, set the uuid
                if lint_mode == LintMode::Fix {
                    group.id = SourceUUID::Uuid(id);
                }
            }
            Err(error) => errors.push(error),
        };

        // Make sure all the fonts are exist and are unique
        let mut fonts = HashMap::<String, ()>::new();
        for font in &group.fonts {
            if fonts.contains_key(font) {
                errors.push(LintErrors::GroupDuplicateFont(
                    ErrorContext::Group(group.name.to_string()),
                    font.to_string(),
                ));
            }

            fonts.insert(font.clone(), ());

            let mut found = false;

            for font_item in &new.fonts {
                if font == &font_item.name {
                    found = true;
                    break;
                }
            }

            if !found {
                errors.push(LintErrors::GroupFontDoesntExist(
                    ErrorContext::Group(group.name.to_string()),
                    font.to_string(),
                ));
            }
        }

        // Sort the fonts by name
        if lint_mode == LintMode::Fix {
            group.fonts.sort_by(|a, b| a.cmp(&b));
        } else {
            // Check if the fonts are sorted
            if !check_sorted(&group.fonts, &|a, b| a < b) {
                errors.push(LintErrors::UnsortedList(ErrorContext::Group(
                    group.name.to_string(),
                )));
            }
        }

        group_names.insert(group.name.clone(), ());
    }

    // Check any fonts with duplicate names or invalid ids
    let mut font_names = HashMap::<String, ()>::new();

    // Sort the fonts by name
    if lint_mode == LintMode::Fix {
        new.fonts.sort_by(|a, b| a.name.cmp(&b.name));
    } else {
        // Check if the groups are sorted
        if !check_sorted(&new.fonts, &|a, b| a.name < b.name) {
            errors.push(LintErrors::UnsortedList(ErrorContext::Fonts));
        }
    }

    let mut downloads: Vec<(ErrorContext, SourceDownload)> = Vec::new();

    for font in &mut new.fonts {
        // Check if the font & short name is valid
        if font_names.contains_key(&font.name) {
            errors.push(LintErrors::DuplicatedName(
                font.name.clone(),
                ErrorContext::Font(font.name.to_string()),
            ));
        }

        if &font.name != &font.short_name && font_names.contains_key(&font.short_name) {
            errors.push(LintErrors::DuplicatedName(
                font.short_name.clone(),
                ErrorContext::Font(font.name.to_string()),
            ));
        }

        if let Err(error) = check_name(&font.name, ErrorContext::Font(font.name.to_string())) {
            errors.push(error);
        }

        if let Err(error) = check_name(&font.short_name, ErrorContext::Font(font.name.to_string()))
        {
            errors.push(error);
        }

        // Check the publisher is valid
        if let Err(error) = check_name(&font.publisher, ErrorContext::Font(font.name.to_string())) {
            errors.push(error);
        }

        // Check Uuid
        match check_or_create_uuid(
            &mut uuids,
            font.id,
            ErrorContext::Font(font.name.to_string()),
            lint_mode,
        ) {
            Ok(id) => {
                // If in fix mode, set the uuid
                if lint_mode == LintMode::Fix {
                    font.id = SourceUUID::Uuid(id);
                }
            }
            Err(error) => errors.push(error),
        };

        // Check if the font has any installations
        if font.installations.len() == 0 {
            errors.push(LintErrors::FontEmpty(ErrorContext::Font(
                font.name.to_string(),
            )));
        }

        // Find all downloads
        for installation in &font.installations {
            let download = match installation {
                SourceInstalationType::Cabextract(data) => &data.download,
            };

            downloads.push((ErrorContext::Font(font.name.to_string()), download.clone()));
        }

        font_names.insert(font.name.clone(), ());
        font_names.insert(font.short_name.clone(), ());
    }

    // Check all the downloads
    for (context, download) in downloads {
        match download {
            SourceDownload::ExternalResource(url) => {
                if url.scheme() != "https" {
                    errors.push(LintErrors::DownloadExternalResourceNotHttps(
                        context.clone(),
                        url.clone(),
                    ));
                }

                // Make the request
                let res = match reqwest::get(url.clone()).await {
                    Ok(response) => response,
                    Err(error) => {
                        error!("Failed to get external resource: {}", error);
                        std::process::exit(1);
                    }
                };

                if !res.status().is_success() {
                    errors.push(LintErrors::DownloadExternalResourceError(
                        context,
                        url.clone(),
                        res.status().as_u16(),
                    ));
                }
            }
            SourceDownload::LocalResource(path) => {
                let path = base_path.join(path);
                if !path.exists() {
                    errors.push(LintErrors::DownloadLocalResourceDoesntExist(context, path));
                }
            }
        }
    }

    (new, errors)
}
