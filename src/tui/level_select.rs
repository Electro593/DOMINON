use color_eyre::eyre::Result;
use crossterm::event::Event;
use ratatui::{
    buffer::Buffer,
    layout::{Rect, Size},
};

use crate::tui::{Screen, screen::ScreenWidget};

#[derive(Clone, Debug)]
pub struct LevelSelectScreen {}

impl LevelSelectScreen {
    pub fn new() -> Self {
        LevelSelectScreen {}
    }
}

impl ScreenWidget for &mut LevelSelectScreen {
    fn try_render(self, area: Rect, buf: &mut Buffer) -> Result<(), Size> {
        let height = 22;
        let width = 78;

        if area.width < width || area.height < height {
            return Err(Size::new(width, height));
        }

        Ok(())
    }

    fn handle_screen_event(self, event: &Event, enhanced_keyboard: bool) -> Result<Option<Screen>> {
        Ok(None)
    }
}
