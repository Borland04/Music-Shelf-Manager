use std::{
    collections::HashMap,
    fs::FileType,
    path::{Path, PathBuf},
};

use ratatui::widgets::{List, ListItem, ListState, StatefulWidget, Widget};

use crate::{AppError, Result};

pub fn from_dir<'b>(path: &Path) -> Result<FsList<'b>> {
    let files =
        std::fs::read_dir(path).map_err(|err| AppError::InitializationFailure(err.to_string()))?;

    let mut list: Vec<PathBuf> = Vec::new();
    for item in files {
        let item = item.map_err(anyhow::Error::new)?;
        let is_file = item
            .file_type()
            .map(|ft| FileType::is_file(&ft))
            .unwrap_or(false);

        if is_file {
            list.push(item.path());
        } else {
            // TODO: move to logging
            // println!(
            //     "Skipping item {} since it's a directory or unknown error occurred",
            //     item.file_name().display()
            // );
            continue;
        }
    }

    Ok(FsList::new(&list))
}

#[derive(Debug)]
pub struct FsList<'a> {
    items: HashMap<String, PathBuf>,
    list: List<'a>,
    list_state: ListState,
}

impl<'a> Default for FsList<'a> {
    fn default() -> Self {
        let list = List::default().highlight_symbol("> ");

        Self {
            items: Default::default(),
            list,
            list_state: Default::default(),
        }
    }
}

impl<'a> FsList<'a> {
    // Init methods
    pub fn new(files: &Vec<PathBuf>) -> Self {
        let result = FsList::default();
        result.items(files)
    }

    pub fn items(mut self, files: &Vec<PathBuf>) -> Self {
        self.items.clear();

        for file in files {
            if let Some(fname) = file.file_name() {
                self.items
                    .insert(fname.to_string_lossy().into_owned(), file.clone());
            }
        }

        let items = self
            .items
            .keys()
            .map(|file| ListItem::new(file.clone()))
            .collect::<Vec<_>>();
        self.list = self.list.items(items);

        if !self.list.is_empty() {
            self.list_state = self.list_state.with_selected(Some(0));
        }

        self
    }

    // Controls
    pub fn select_prev(&mut self) {
        self.list_state.select_previous();
    }

    pub fn select_next(&mut self) {
        self.list_state.select_next();
    }
}

impl<'a> Widget for &mut FsList<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        StatefulWidget::render(&self.list, area, buf, &mut self.list_state);
    }
}
