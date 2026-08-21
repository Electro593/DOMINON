// use std::slice::Iter;

use crossterm::event::{Event, KeyCode, KeyEventKind, MouseButton, MouseEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph, Widget},
};

use crate::tui::screen::screen_style;

// use crate::util::tree::Tree;

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

    pub fn press(&mut self) {
        self.pressed = true;
    }

    pub fn cancel(&mut self) {
        self.pressed = false;
    }

    pub fn click(&mut self) {
        self.pressed = false;
        self.clicked = true;
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
            Event::Key(key_event) if focused => match key_event.kind {
                KeyEventKind::Press | KeyEventKind::Repeat => {
                    if self.key == Some(key_event.code) {
                        self.click();
                        return true;
                    }

                    match key_event.code {
                        KeyCode::Esc => {
                            if self.pressed {
                                self.pressed = false;
                                return true;
                            }
                        }
                        KeyCode::Enter | KeyCode::Char(' ') => {
                            if enhanced_keyboard {
                                self.press();
                            } else {
                                self.click();
                            }
                            return true;
                        }
                        _ => {}
                    }
                }
                KeyEventKind::Release => match key_event.code {
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        if self.pressed {
                            self.click();
                            return true;
                        }
                    }
                    _ => {}
                },
            },
            Event::Mouse(mouse_event) => {
                let hit = self.is_hit(mouse_event.column, mouse_event.row);
                self.hovered = hit;

                match mouse_event.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        if hit {
                            self.press();
                            return true;
                        }
                    }
                    MouseEventKind::Up(MouseButton::Left) => {
                        self.cancel();
                        if focused && hit {
                            self.click();
                            return true;
                        }
                    }
                    _ => {}
                }
            }
            Event::Resize(_, _) => self.hovered = false,
            _ => {}
        };

        false
    }
}

pub fn make_button<'a>(
    state: &'a mut ButtonState,
    focused: bool,
    text: String,
    key_index: Option<usize>,
) -> Button<'a, Paragraph<'a>> {
    let chars = &mut text.chars();

    let mut style = screen_style();
    let mut block = Block::bordered();

    let mut prefix = None;
    let first = chars
        .take(key_index.unwrap_or(text.len()))
        .collect::<String>();
    let key = chars.next();
    let rest = chars.collect::<String>();

    if state.hovered {
        style = Style::new().green();
        block = block.border_style(style);
    }

    if focused {
        prefix = Some(Span::raw("> "));
        block = block.border_type(BorderType::Double);
    }

    if state.pressed {
        block = block.border_type(BorderType::Thick);
    }

    let spans = vec![
        prefix,
        Some(Span::raw(first)),
        key.map(|c| Span::raw(c.to_string()).magenta()),
        Some(Span::raw(rest)),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<Span>>();
    let line = Line::from(spans).style(style);
    let mut button = Button::new(Paragraph::new(line).centered().block(block), state);

    if let Some(c) = key {
        button = button.key(KeyCode::Char(c.to_ascii_lowercase()));
    }

    button
}

// pub trait FocusWidget: Sized {
//     fn render(&self, area: Rect, buf: &mut Buffer) -> Rect;
// }
//
// pub struct FocusTree<T> {
//     items: Vec<(T, usize)>,
//     tree: Tree<usize>,
//     focus_index: Option<usize>,
// }
//
// impl<T> FocusTree<T> {
//     pub fn get_focused(&self) -> Option<&T> {
//         Some(&self.items[self.focus_index?].0)
//     }
//
//     pub fn is_focused(&self, value: &T) -> bool {
//         self.get_focused().map_or(false, |f| std::ptr::eq(f, value))
//     }
//
//     pub fn focus(&mut self, focus: Option<usize>) {
//         self.focus_index = focus;
//     }
// }
//
// impl<T: FocusWidget> Widget for FocusTree<T> {
//     fn render(self, area: Rect, buf: &mut Buffer) {
//         let mut stack = vec![(self.tree.root(), area)];
//
//         while let Some((node, parent_area)) = stack.pop() {
//             let item = &self.items[node.index].0;
//             let inner_area = item.render(parent_area, buf);
//             node.next().map(|n| stack.push((n, inner_area)));
//             node.child(0).map(|n| stack.push((n, inner_area)));
//         }
//     }
// }
//
// impl<T: EventHandler> EventHandler for FocusTree<T> {
//     fn handle_event(&mut self, event: &Event, focused: bool, enhanced_keyboard: bool) -> bool {
//         for index in self.tree.root().iter_post_order() {
//             let is_focused = focused && self.is_focused(&self.items[*index].0);
//             let item = &mut self.items[*index].0;
//             if item.handle_event(&event, is_focused, enhanced_keyboard) {
//                 self.focus_index = Some(*index);
//                 return true;
//             }
//         }
//
//         match event {
//             Event::Key(key_event) => match key_event.kind {
//                 KeyEventKind::Press | KeyEventKind::Repeat => match key_event.code {
//                     KeyCode::Esc => {
//                         self.focus(None);
//                         return true;
//                     }
//                     _ => {}
//                 },
//                 _ => {}
//             },
//             _ => {}
//         }
//
//         false
//     }
// }
