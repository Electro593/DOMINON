use std::fs::read_dir;

use color_eyre::eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind, MouseEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Rect, Size},
    widgets::Widget,
};

use crate::tui::{
    Screen,
    screen::ScreenWidget,
    widget::{ButtonState, EventHandler, make_button},
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
    focus: bool,
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
            focus: true,
        }
    }

    fn select_prev(&mut self) {
        self.selected = self.selected.map_or_else(
            || Some(1),
            |i| {
                let next = (i + (2 - 1)) % 2;
                self.selected = Some(next);
                Some(next)
            },
        );
    }

    fn select_next(&mut self) {
        self.selected = self.selected.map_or_else(
            || Some(0),
            |i| {
                let next = (i + 1) % 2;
                self.selected = Some(next);
                Some(next)
            },
        );
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
            let mut text = option.name.clone();
            if text.len() > 20 {
                text = format!("{}...", text[0..17].to_string());
            }

            let button_width = text.len() as u16 + 4;
            let button_height = 3;
            if across + button_width >= width {
                across = 0;
                down += button_height;
            }

            make_button(
                &mut option.state,
                self.focus && self.selected == Some(i),
                text,
                None,
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
        for (i, option) in self.levels.iter_mut().flatten().enumerate() {
            let focused = self.selected == Some(i);
            let consume = option.state.handle_event(event, focused, enhanced_keyboard);
            if consume {
                self.selected = Some(i);
                return Ok(None);
            }
        }

        self.handle_event(event, self.focus, enhanced_keyboard);
        Ok(None)
    }
}

impl EventHandler for LevelSelectScreen {
    fn handle_event(&mut self, event: &Event, focused: bool, _: bool) -> bool {
        match event {
            Event::Key(key_event) if focused => match key_event.kind {
                KeyEventKind::Press | KeyEventKind::Repeat => match key_event.code {
                    KeyCode::Up => self.select_prev(),
                    KeyCode::Down => self.select_next(),
                    KeyCode::Esc => {
                        if self.selected.is_some() {
                            self.selected = None;
                        } else {
                            self.focus = false;
                        }
                    }
                    _ => {}
                },
                _ => {}
            },
            Event::Mouse(mouse_event) if focused => match mouse_event.kind {
                MouseEventKind::ScrollDown => self.select_next(),
                MouseEventKind::ScrollUp => self.select_prev(),
                _ => {}
            },
            Event::FocusLost => self.focus = false,
            Event::FocusGained => self.focus = true,
            _ => {}
        };
        false
    }
}
