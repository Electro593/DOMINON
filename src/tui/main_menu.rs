use color_eyre::eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind, MouseButton, MouseEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect, Size},
    style::Style,
    widgets::{Block, BorderType, Paragraph, StatefulWidget, Widget},
};
use tui_big_text::{BigText, PixelSize};
use tui_widget_list::{ListBuilder, ListState, ListView, ScrollDirection, hit_test::Hit};

use crate::tui::{Screen, screen::ScreenWidget};

#[derive(Clone, Debug)]
pub struct MainMenuScreen {
    focus: bool,
    list_state: ListState,
}

impl MainMenuScreen {
    pub fn new() -> Self {
        MainMenuScreen {
            focus: true,
            list_state: ListState::new_with_index(Some(0)),
        }
    }
}

impl ScreenWidget for &mut MainMenuScreen {
    fn try_render(self, area: Rect, buf: &mut Buffer) -> Result<(), Size> {
        let height = 19;
        let width = 78;

        if area.width < width || area.height < height {
            return Err(Size::new(width, height));
        }

        let horizontal_frame_layout = Layout::horizontal([Constraint::Length(width)])
            .flex(Flex::Center)
            .split(area);
        let frame_layout = Layout::vertical([Constraint::Length(height)])
            .flex(Flex::Center)
            .split(horizontal_frame_layout[0]);

        let center_layout = Layout::vertical([
            Constraint::Length(5),
            Constraint::Length(8),
            Constraint::Fill(1),
        ])
        .split(frame_layout[0]);

        // Title top text
        BigText::builder()
            .pixel_size(PixelSize::HalfHeight)
            .style(Style::new().dark_gray())
            .lines(vec!["Welcome to".into()])
            .centered()
            .build()
            .render(center_layout[0], buf);

        // Title bottom text
        BigText::builder()
            .pixel_size(PixelSize::Full)
            .style(Style::new().white())
            .lines(vec!["DOMINON!".into()])
            .centered()
            .build()
            .render(center_layout[1], buf);

        // Button list
        ListView::new(
            ListBuilder::new(|context| {
                let is_selected = context.is_selected && self.focus;

                let selected_text = |text: &str| {
                    if is_selected {
                        format!("> {text}")
                    } else {
                        String::from(text)
                    }
                };

                let mut item = match context.index {
                    0 => Paragraph::new(selected_text("Level Select")),
                    _ => Paragraph::new(selected_text("Quit")),
                };

                let mut block = Block::bordered();
                if is_selected {
                    block = block.border_type(BorderType::Double);
                }

                item = item.centered().block(block);
                return (item, 3);
            }),
            2,
        )
        .scroll_direction(ScrollDirection::Forward)
        .infinite_scrolling(true)
        .render(center_layout[2], buf, &mut self.list_state);

        Ok(())
    }

    fn handle_event(self, event: Event, enhanced_keyboard: bool) -> Result<Option<Screen>> {
        let screens = [Screen::LevelSelect, Screen::None];

        let confirm = |i: Option<usize>| {
            if self.focus && i.is_some() {
                Ok(Some(screens[i.unwrap()].clone()))
            } else {
                Ok(None)
            }
        };

        match event {
            Event::Key(key_event) if self.focus => match key_event.kind {
                KeyEventKind::Press | KeyEventKind::Repeat => match key_event.code {
                    KeyCode::Up => self.list_state.previous(),
                    KeyCode::Down => self.list_state.next(),
                    KeyCode::Esc => self.list_state.select(None),
                    KeyCode::Enter | KeyCode::Char(' ') if !enhanced_keyboard => {
                        return confirm(self.list_state.selected);
                    }
                    _ => {}
                },
                KeyEventKind::Release => match key_event.code {
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        return confirm(self.list_state.selected);
                    }
                    _ => {}
                },
            },
            Event::Mouse(mouse_event) if self.focus => match mouse_event.kind {
                MouseEventKind::ScrollDown => self.list_state.next(),
                MouseEventKind::ScrollUp => self.list_state.previous(),
                MouseEventKind::Down(_) | MouseEventKind::Up(_) => {
                    let hit = self
                        .list_state
                        .hit_test(mouse_event.column, mouse_event.row)
                        .and_then(|h| match h {
                            Hit::Area => None,
                            Hit::Item(index) => Some(index),
                        });

                    match mouse_event.kind {
                        MouseEventKind::Down(MouseButton::Left) => {
                            self.list_state.select(hit.or(Some(0)))
                        }
                        MouseEventKind::Up(MouseButton::Left) => return confirm(hit),
                        _ => {}
                    }
                }
                _ => {}
            },
            Event::FocusLost => self.focus = false,
            Event::FocusGained => self.focus = true,
            _ => {}
        }
        Ok(None)
    }
}
