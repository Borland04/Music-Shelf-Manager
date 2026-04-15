use std::path::PathBuf;

use crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, Padding, Paragraph, Widget};
use ratatui::DefaultTerminal;

use crate::ui::fs_list::{from_dir, FsList};

use crate::{AppError, Result};

pub struct App<'a> {
    working_dir: PathBuf,
    is_closed: bool,
    list: FsList<'a>,
}

impl<'a> App<'a> {
    pub fn new() -> Result<Self> {
        let cdir = std::env::current_dir().map_err(|err| {
            AppError::InitializationFailure(format!("Error while getting current dir: {}", err))
        })?;
        let list_widget = from_dir(&cdir)?;

        Ok(Self {
            working_dir: cdir,
            is_closed: false,
            list: list_widget,
        })
    }

    pub fn run(mut self) -> Result<()> {
        ratatui::run(|term| {
            while !self.is_closed {
                self.tick(term)?;
                self.handle_events()?;
            }

            Ok(())
        })
    }

    fn tick(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        terminal
            .draw(|frame| {
                self.render(frame.area(), frame.buffer_mut());
            })
            .map_err(|err| {
                anyhow::Error::msg(format!(
                    "Unexpected error during drawing into terminal: {}",
                    err
                ))
            })?;
        Ok(())
    }

    fn handle_events(&mut self) -> Result<()> {
        if let Some(key_event) = crossterm::event::read()?.as_key_event() {
            match key_event.code {
                KeyCode::Char('j') | KeyCode::Down => {
                    self.list.select_next();
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.list.select_prev();
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.is_closed = true;
                }
                _ => {}
            }
        }

        Ok(())
    }
}

impl<'a> Widget for &mut App<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        // Layouts
        let root_layout = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).spacing(1);
        let [title_area, main_area] = area.layout(&root_layout);

        let block_title =
            Line::from(format!(" Music in {} ", self.working_dir.to_string_lossy())).centered();
        let block_footer =
            Line::from(" J/↓ to move down; K/↑ to move up; Esc/q to exit ").right_aligned();
        let list_block = Block::bordered()
            .title(block_title)
            .title_bottom(block_footer)
            .padding(Padding::symmetric(2, 2));
        let list_area = list_block.inner(main_area);

        // Widgets in these layouts
        let title = Line::from("Music Shelf Manager").centered();

        title.render(title_area, buf);
        list_block.render(main_area, buf);
        self.list.render(list_area, buf);
    }
}
