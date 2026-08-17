use std::fs::read_dir;

use color_eyre::eyre::Result;
use crossterm::event::Event;
use ratatui::{
    buffer::Buffer,
    layout::{Rect, Size},
    style::{Style, Styled},
    widgets::{Block, BorderType, Paragraph, Shadow, Widget},
};

use crate::tui::{
    Screen,
    screen::{ScreenWidget, screen_style},
    widget::{Button, ButtonState, EventHandler},
};

#[derive(Debug)]
struct LevelOption {
    name: String,
    state: ButtonState,
}

#[derive(Debug)]
pub struct LevelSelectScreen {
    levels: Result<Vec<LevelOption>>,
    selected: Option<usize>,
}

impl LevelSelectScreen {
    pub fn new() -> Self {
        let levels = read_dir("levels")
            .map(|paths| {
                paths
                    .flatten()
                    .filter_map(|dir_entry| {
                        let mut path = dir_entry.path();
                        (path.is_file() && path.set_extension(""))
                            .then_some(())
                            .and_then(|_| path.file_name())
                            .map(|f| LevelOption {
                                name: f.to_string_lossy().to_string(),
                                state: ButtonState::new(),
                            })
                    })
                    .collect::<Vec<LevelOption>>()
            })
            .map_err(|e| e.into());

        LevelSelectScreen {
            levels,
            selected: Some(0),
        }
    }
}

impl ScreenWidget for &mut LevelSelectScreen {
    fn try_render(self, area: Rect, buf: &mut Buffer) -> Result<(), Size> {
        let height = 19;
        let width = 78;

        if area.width < width || area.height < height {
            return Err(Size::new(width, height));
        }

        let mut across = 0;
        let mut down = 0;

        for (i, option) in self.levels.iter_mut().flatten().enumerate() {
            let mut name = option.name.clone();
            if name.len() > 20 {
                name = format!("{}...", name[0..17].to_string());
            }

            let mut block = Block::bordered();

            if option.state.hovered {
                block = block.border_style(Style::new().green());
            }

            if let Some(selected) = self.selected
                && selected == i
            {
                name = format!("> {name}");
                block = block.border_type(BorderType::Double);
            }

            let button_width = name.len() as u16 + 4;
            let button_height = 3;
            if across + button_width >= width {
                across = 0;
                down += button_height;
            }

            Button::new(
                Paragraph::new(name)
                    .centered()
                    .block(block)
                    .style(screen_style()),
                &mut option.state,
            )
            .render(
                Rect::new(area.x + across, area.y + down, button_width, button_height),
                buf,
            );

            across += button_width;
        }

        Ok(())
    }

    fn handle_screen_event(self, event: &Event, enhanced_keyboard: bool) -> Result<Option<Screen>> {
        for option in self.levels.iter_mut().flatten() {
            option.state.handle_event(event, enhanced_keyboard);
        }
        Ok(None)
    }
}
