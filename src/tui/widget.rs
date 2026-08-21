use crossterm::event::{Event, KeyCode, KeyEventKind, MouseButton, MouseEventKind};
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use crate::util::tree::{Tree};

pub trait EventHandler {
    fn handle_event(&mut self, event: &Event, focused: bool, enhanced_keyboard: bool) -> bool;
}

#[derive(Clone, Debug)]
pub struct ButtonState {
    key: Option<KeyCode>,
    pub area: Rect,
    pub hovered: bool,
    pub pressed: bool,
    pub clicked: bool,
}

impl ButtonState {
    pub fn new() -> Self {
        Self {
            key: None,
            area: Rect::ZERO,
            hovered: false,
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

impl EventHandler for ButtonState {
    fn handle_event(&mut self, event: &Event, focused: bool, enhanced_keyboard: bool) -> bool {
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

                    if focused {
                        match key_event.code {
                            KeyCode::Esc => {
                                if self.pressed {
                                    self.pressed = false;
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
            Event::Mouse(mouse_event) => {
                let hit = self.is_hit(mouse_event.column, mouse_event.row);
                self.hovered = hit;

                if hit {
                    match mouse_event.kind {
                        MouseEventKind::Down(MouseButton::Left) => self.press(true),
                        MouseEventKind::Up(MouseButton::Left) => self.press(false),
                        _ => {}
                    }
                }
            }
            Event::Resize(_, _) => self.hovered = false,
            _ => {}
        };

        false
    }
}

struct WidgetTree {
    tree: Tree<Box<dyn Widget>>,
    focus_index: Option<usize>
}

impl WidgetTree {
    fn focus(&mut self, focus: Option<usize>) {
        self.focus_index = focus;
    }
}