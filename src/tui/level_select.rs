use std::fs::read_dir;

use color_eyre::eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers, MouseEventKind};
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

#[derive(Clone, Debug)]
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
                    .chain((0..100).map(|i| LevelOption {
                        name: format!("Duplicant {i}"),
                        state: ButtonState::new(),
                    }))
                    .collect::<Vec<LevelOption>>()
            })
            .map_err(|e| e.into());

        LevelSelectScreen {
            levels,
            selected: Some(0),
            focus: true,
        }
    }

    fn selection_bounds(&self) -> Option<Rect> {
        self.levels
            .iter()
            .flatten()
            .map(|level| level.state.area)
            .reduce(|a, b| a.union(b))
    }

    fn select_hit(&mut self, x: u16, y: u16) -> bool {
        for (i, level) in self.levels.iter_mut().flatten().enumerate() {
            if level.state.is_hit(x, y) {
                self.selected = Some(i);
                return true;
            }
        }
        false
    }

    fn select_hit_walk(&mut self, min_x: u16, mut x: u16, y: u16) -> bool {
        while x >= min_x {
            if self.select_hit(x, y) {
                return true;
            }

            if x < 5 {
                break;
            }

            x -= 5;
        }
        false
    }

    fn select_up(&mut self) {
        if let Ok(levels) = &self.levels {
            if let Some(bounds) = self.selection_bounds() {
                let max_y = bounds.bottom() - 1;
                match self.selected {
                    None => {
                        self.select_hit(bounds.x, max_y);
                    }
                    Some(i) => {
                        let area = levels[i].state.area;
                        let y = if area.y == bounds.y {
                            max_y
                        } else {
                            area.y - 1
                        };
                        self.select_hit_walk(bounds.x, area.x + area.width / 2, y);
                    }
                };
            }
        }
    }

    fn select_down(&mut self) {
        if let Ok(levels) = &self.levels {
            if let Some(bounds) = self.selection_bounds() {
                match self.selected {
                    None => {
                        self.select_hit(bounds.x, bounds.y);
                    }
                    Some(i) => {
                        let area = levels[i].state.area;
                        let x = area.x + area.width / 2;
                        let mut y = area.bottom();
                        if y >= bounds.bottom() {
                            y = bounds.top();
                        }
                        self.select_hit_walk(bounds.x, x, y);
                    }
                };
            }
        }
    }

    fn select_left(&mut self) {
        if let Ok(levels) = &self.levels {
            if let Some(bounds) = self.selection_bounds() {
                let max_x = bounds.right() - 5;
                match self.selected {
                    None => {
                        self.select_hit_walk(bounds.x, max_x, bounds.y);
                    }
                    Some(i) => {
                        let area = levels[i].state.area;
                        let x = if area.x == bounds.x {
                            max_x
                        } else {
                            area.x - 1
                        };
                        self.select_hit_walk(bounds.x, x, area.y);
                    }
                };
            }
        }
    }

    fn select_right(&mut self) {
        if let Ok(levels) = &self.levels {
            if let Some(bounds) = self.selection_bounds() {
                match self.selected {
                    None => {
                        self.select_hit(bounds.x, bounds.y);
                    }
                    Some(i) => {
                        let area = levels[i].state.area;
                        if !self.select_hit(area.x + area.width, area.y) {
                            self.select_hit(bounds.x, area.y);
                        }
                    }
                };
            }
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

        // TODO: Error message
        // TODO: Back button
        // TODO: Scroll the list properly
        // TODO: Input name directly

        for (i, option) in self.levels.iter_mut().flatten().enumerate() {
            let is_selected = self.focus && self.selected == Some(i);

            let mut text = option.name.clone();
            if text.len() > 20 {
                text = format!("{}...", text[0..17].to_string());
            }

            let button_width = text.len() as u16 + 6;
            let button_height = 3;
            if across + button_width >= area.width {
                across = 0;
                down += button_height;
            }

            make_button(&mut option.state, is_selected, text, None).render(
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
                    KeyCode::Up => self.select_up(),
                    KeyCode::Down => self.select_down(),
                    KeyCode::Left => self.select_left(),
                    KeyCode::Right => self.select_right(),
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
                MouseEventKind::ScrollUp => {
                    if mouse_event.modifiers.contains(KeyModifiers::SHIFT) {
                        self.select_left();
                    } else {
                        self.select_up();
                    }
                }
                MouseEventKind::ScrollDown => {
                    if mouse_event.modifiers.contains(KeyModifiers::SHIFT) {
                        self.select_right();
                    } else {
                        self.select_down();
                    }
                }
                MouseEventKind::ScrollLeft => self.select_left(),
                MouseEventKind::ScrollRight => self.select_right(),
                _ => {}
            },
            Event::FocusLost => self.focus = false,
            Event::FocusGained => self.focus = true,
            _ => {}
        };
        false
    }
}
