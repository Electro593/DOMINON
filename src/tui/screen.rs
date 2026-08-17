use color_eyre::eyre::Result;
use crossterm::event::Event;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect, Size},
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{Block, Clear, Paragraph, Widget, Wrap},
};

use crate::tui::Screen;

pub trait ScreenWidget: Sized {
    fn handle_screen_event(self, event: &Event, enhanced_keyboard: bool) -> Result<Option<Screen>>;
    fn try_render(self, area: Rect, buf: &mut Buffer) -> Result<(), Size>;
}

pub struct ScreenWrapper<T>(pub T);

impl<T> Widget for ScreenWrapper<T>
where
    T: ScreenWidget,
{
    fn render(self, area: Rect, buf: &mut Buffer) {
        let frame_block = Block::bordered()
            .title(Line::from("DOMINON!".bold()).left_aligned())
            .title_bottom(
                Line::from(format!("v{}", env!("CARGO_PKG_VERSION")).italic()).right_aligned(),
            )
            .border_set(border::THICK);

        let inner = frame_block.inner(area);
        frame_block.render(area, buf);

        if let Err(min_size) = self.0.try_render(inner, buf) {
            let text = format!(
                "Terminal is too small! Must be at least {}x{}.",
                min_size.width, min_size.height
            );

            let wrapped_height = textwrap::wrap(text.as_str(), inner.width as usize).len() as u16;

            let layout = Layout::vertical([Constraint::Length(wrapped_height)])
                .flex(Flex::Center)
                .split(inner);

            Clear.render(layout[0], buf);
            Paragraph::new(text)
                .wrap(Wrap { trim: true })
                .centered()
                .render(layout[0], buf);
        }
    }
}
