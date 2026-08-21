use color_eyre::eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind, MouseEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect, Size},
    style::Style,
    widgets::Widget,
};
use tui_big_text::{BigText, PixelSize};

use crate::tui::{
    Screen,
    level_select::LevelSelectScreen,
    screen::ScreenWidget,
    widget::{ButtonState, EventHandler, make_button},
};

#[derive(Debug)]
pub struct MainMenuScreen {
    focus: bool,
    selected: Option<usize>,
    button_state: [ButtonState; 2],
}

impl MainMenuScreen {
    pub fn new() -> Self {
        MainMenuScreen {
            focus: true,
            selected: Some(0),
            button_state: [ButtonState::new(), ButtonState::new()],
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

impl EventHandler for MainMenuScreen {
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
            Constraint::Length(3),
            Constraint::Length(3),
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

        make_button(
            &mut self.button_state[0],
            self.focus && self.selected == Some(0),
            "Level Select".into(),
            Some(0),
        )
        .render(center_layout[2], buf);
        make_button(
            &mut self.button_state[1],
            self.focus && self.selected == Some(1),
            "Quit".into(),
            Some(0),
        )
        .render(center_layout[3], buf);

        Ok(())
    }

    fn handle_screen_event(self, event: &Event, enhanced_keyboard: bool) -> Result<Option<Screen>> {
        let mut press_button = |i| {
            let focused = self.focus && self.selected == Some(i);
            if self.button_state[i].handle_event(event, focused, enhanced_keyboard) {
                self.selected = Some(i);
                true
            } else {
                false
            }
        };

        let _ = press_button(0)
            || press_button(1)
            || self.handle_event(event, self.focus, enhanced_keyboard);

        if self.button_state[0].clicked {
            return Ok(Some(Screen::LevelSelect(LevelSelectScreen::new())));
        }

        if self.button_state[1].clicked {
            return Ok(Some(Screen::None));
        }

        Ok(None)
    }
}
