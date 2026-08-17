use crossterm::event::{Event, KeyCode, KeyEventKind, MouseButton, MouseEventKind};
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

pub trait EventHandler {
    fn handle_event(&mut self, event: &Event, enhanced_keyboard: bool) -> bool;
}

fn hit_test(area: Rect, column: u16, row: u16) -> bool {
    area.left() <= column && column < area.right() && row >= area.top() && row < area.bottom()
}

pub trait Hoverable {
    fn hover(&mut self, hovered: bool);

    fn handle_hover(&mut self, area: Rect, event: &Event) {
        match event {
            Event::Mouse(mouse_event) => {
                self.hover(hit_test(area, mouse_event.column, mouse_event.row));
            }
            Event::Resize(_, _) => self.hover(false),
            _ => {}
        }
    }
}

pub trait Focusable {
    fn focused(&self) -> bool;
    fn focus(&mut self, focused: bool);

    fn handle_focus(&mut self, event: &Event) {
        let focused = self.focused();
        match event {
            Event::Key(key_event) if focused => match key_event.code {
                KeyCode::Esc => match key_event.kind {
                    KeyEventKind::Press | KeyEventKind::Repeat => {
                        self.focus(false);
                    }
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        };
    }
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

impl Hoverable for ButtonState {
    fn hover(&mut self, hovered: bool) {
        self.hovered = hovered;
    }
}

impl Focusable for ButtonState {
    fn focused(&self) -> bool {
        self.focused
    }

    fn focus(&mut self, focused: bool) {
        self.focused = focused;
    }
}

impl EventHandler for ButtonState {
    fn handle_event(&mut self, event: &Event, enhanced_keyboard: bool) -> bool {
        self.handle_hover(self.area, event);

        match event {
            Event::Key(key_event) => match key_event.kind {
                KeyEventKind::Press | KeyEventKind::Repeat => {
                    if let KeyEventKind::Press = key_event.kind
                        && self.key.is_some()
                        && key_event.code == self.key.unwrap()
                    {
                        self.press(true);
                        self.press(false);
                        return true;
                    }

                    if self.focused {
                        match key_event.code {
                            KeyCode::Esc => {
                                if self.pressed {
                                    self.cancel();
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
                }
                KeyEventKind::Release => match key_event.code {
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        self.press(false);
                        return true;
                    }
                    _ => {}
                },
            },
            Event::Mouse(mouse_event) => match mouse_event.kind {
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
            },
            _ => {}
        };

        self.handle_focus(event);

        false
    }
}

#[derive(Clone, Debug)]
pub struct GridState<T> {
    area: Rect,
    children: T,
    width: usize,
    focused: bool,
}

impl<T> EventHandler for GridState<T> {
    fn handle_event(&mut self, event: &Event, enhanced_keyboard: bool) -> bool {
        match event {
            Event::Key(key_event) => match key_event.kind {
                KeyEventKind::Press | KeyEventKind::Repeat => match key_event.code {
                    KeyCode::Up => {}
                    KeyCode::Down => {}
                    KeyCode::Left => {}
                    KeyCode::Right => {}
                    KeyCode::Tab => {}
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        };
        false
    }
}
