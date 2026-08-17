use crossterm::event::{Event, KeyCode, KeyEventKind, MouseButton, MouseEventKind};
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

pub trait EventHandler {
    fn handle_event(&mut self, event: &Event, enhanced_keyboard: bool) -> bool;
}

#[derive(Clone, Debug)]
pub struct ButtonState {
    key: Option<KeyCode>,
    pub area: Rect,
    pub hovered: bool,
    pub focused: bool,
    pub pressed: bool,
    pub clicked: bool,
}

impl ButtonState {
    pub fn new() -> Self {
        Self {
            key: None,
            area: Rect::ZERO,
            hovered: false,
            focused: false,
            pressed: false,
            clicked: false,
        }
    }

    fn is_hit(&self, column: u16, row: u16) -> bool {
        self.area.left() <= column
            && column < self.area.right()
            && row >= self.area.top()
            && row < self.area.bottom()
    }

    pub fn hover(&mut self, hovered: bool) {
        self.hovered = hovered;
    }

    pub fn focus(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn press(&mut self, pressed: bool) {
        if self.pressed && !pressed {
            self.clicked = true;
        }
        self.pressed = pressed;
    }

    pub fn cancel(&mut self) {
        self.pressed = false;
    }
}

pub struct Button<'a, T> {
    pub content: T,
    pub state: &'a mut ButtonState,
}

impl<'a, T> Button<'a, T> {
    #[must_use]
    pub fn new(content: T, state: &'a mut ButtonState) -> Self {
        state.clicked = false;
        Self { content, state }
    }

    pub fn key(self, code: KeyCode) -> Self {
        self.state.key = Some(code);
        self
    }
}

impl<T: Widget> Widget for Button<'_, T> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.state.area = area;
        self.content.render(area, buf);
    }
}

impl EventHandler for &mut ButtonState {
    fn handle_event(&mut self, event: &Event, enhanced_keyboard: bool) -> bool {
        match event {
            Event::Key(key_event) if self.focused => match key_event.kind {
                KeyEventKind::Press | KeyEventKind::Repeat => {
                    if let KeyEventKind::Press = key_event.kind
                        && self.key.is_some()
                        && key_event.code == self.key.unwrap()
                    {
                        self.press(true);
                        self.press(false);
                        return true;
                    }

                    match key_event.code {
                        KeyCode::Esc => {
                            if self.pressed {
                                self.cancel();
                                return true;
                            } else if self.focused {
                                self.focus(false);
                                return true;
                            }
                        }
                        KeyCode::Enter | KeyCode::Char(' ') => {
                            self.press(true);
                            if !enhanced_keyboard {
                                self.press(false);
                            }
                            return true;
                        }
                        _ => {}
                    }
                }
                KeyEventKind::Release => match key_event.code {
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        self.press(false);
                        return true;
                    }
                    _ => {}
                },
            },
            Event::Mouse(mouse_event) => {
                self.hover(self.is_hit(mouse_event.column, mouse_event.row));

                match mouse_event.kind {
                    MouseEventKind::Down(button) => {
                        let hit = self.is_hit(mouse_event.column, mouse_event.row);
                        self.focus(hit);
                        if let MouseButton::Left = button
                            && hit
                        {
                            self.press(true);
                        }
                    }
                    MouseEventKind::Up(MouseButton::Left) => {
                        if self.is_hit(mouse_event.column, mouse_event.row) {
                            self.press(false);
                        }
                    }
                    _ => {}
                }
            }
            Event::Resize(_, _) => self.hover(false),
            _ => {}
        }
        false
    }
}
