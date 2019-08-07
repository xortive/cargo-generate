use crate::config::TemplateConfig;
use ignore::gitignore::GitignoreBuilder;
use ignore::Match;
use std::iter::Iterator;
use std::path::Path;
use walkdir::{DirEntry, Result as WDResult, WalkDir};

pub fn create_matcher<'a>(
    template_config: &'a TemplateConfig,
    project_dir: &'a Path,
) -> Result<impl Iterator<Item = WDResult<DirEntry>> + 'a, failure::Error> {
    let mut include_builder = GitignoreBuilder::new(project_dir);
    for rule in template_config.include.clone().unwrap_or_else(Vec::new) {
        include_builder.add_line(None, &rule)?;
    }
    let include_matcher = include_builder.build()?;

    let mut exclude_builder = GitignoreBuilder::new(project_dir);
    for rule in template_config.exclude.clone().unwrap_or_else(Vec::new) {
        exclude_builder.add_line(None, &rule)?;
    }
    let exclude_matcher = exclude_builder.build()?;

    let should_include = move |relative_path: &Path| -> bool {
        // "Include" and "exclude" options are mutually exclusive.
        // if no include is made, we will default to ignore_exclude
        // which if there is no options, matches everything
        if template_config.include.is_none() {
            match exclude_matcher
                .matched_path_or_any_parents(relative_path, /* is_dir */ false)
            {
                Match::None => true,
                Match::Ignore(_) => false,
                Match::Whitelist(_) => true,
            }
        } else {
            match include_matcher
                .matched_path_or_any_parents(relative_path, /* is_dir */ false)
            {
                Match::None => false,
                Match::Ignore(_) => true,
                Match::Whitelist(_) => false,
            }
        }
    };

    fn is_dir(entry: &DirEntry) -> bool {
        entry.file_type().is_dir()
    }

    fn is_git_metadata(entry: &DirEntry) -> bool {
        entry
            .path()
            .to_str()
            .map(|s| s.contains(".git"))
            .unwrap_or(false)
    }

    Ok(WalkDir::new(project_dir).into_iter().filter(move |e| {
        if let Ok(e) = e {
            let relative_path = e
                .path()
                .strip_prefix(project_dir)
                .expect("strip project dir before matching");
            !is_dir(e) && !is_git_metadata(e) && should_include(relative_path)
        } else {
            false
        }
    }))
}

pub fn create_default_matcher(
    project_dir: &Path,
) -> Result<impl Iterator<Item = WDResult<DirEntry>>, failure::Error> {
    fn is_dir(entry: &DirEntry) -> bool {
        entry.file_type().is_dir()
    }

    fn is_git_metadata(entry: &DirEntry) -> bool {
        entry
            .path()
            .to_str()
            .map(|s| s.contains(".git"))
            .unwrap_or(false)
    }

    Ok(WalkDir::new(project_dir).into_iter().filter(move |e| {
        if let Ok(e) = e {
            !is_dir(e) && !is_git_metadata(e)
        } else {
            false
        }
    }))
}