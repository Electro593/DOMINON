use std::{io::stdout, time::Duration};

use color_eyre::eyre::{Context, Result};
use crossterm::{
    event::{
        self, DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture,
        Event, KeyCode, KeyModifiers, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags,
        PushKeyboardEnhancementFlags,
    },
    execute,
};
use ratatui::{DefaultTerminal, Frame};

use crate::{
    board::Board,
    tui::{
        main_menu::MainMenuScreen,
        screen::{ScreenWidget, ScreenWrapper},
    },
};

mod level_select;
mod main_menu;
mod screen;
mod widget;

#[derive(Clone, Debug)]
enum Screen {
    None,
    MainMenu(MainMenuScreen),
    LevelSelect,
    Level(Board),
}

#[derive(Debug)]
struct App {
    enhanced_keyboard: bool,
    screen: Screen,
}

impl App {
    fn new(enhanced_keyboard: bool) -> Self {
        App {
            enhanced_keyboard,
            screen: Screen::MainMenu(MainMenuScreen::new()),
        }
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !matches!(self.screen, Screen::None) {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events().wrap_err("handle events failed")?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        match &mut self.screen {
            Screen::MainMenu(menu) => frame.render_widget(ScreenWrapper(menu), frame.area()),
            _ => {}
        }
    }

    fn handle_events(&mut self) -> Result<()> {
        while event::poll(Duration::from_millis(1))? {
            let event = event::read()?;

            if let Event::Key(key) = event {
                if let KeyCode::Char('c') = key.code {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        self.screen = Screen::None;
                        return Ok(());
                    }
                }
            }

            let new_screen = match &mut self.screen {
                Screen::MainMenu(screen) => {
                    screen.handle_screen_event(&event, self.enhanced_keyboard)
                }
                _ => Ok(None),
            }?;

            if let Some(screen) = new_screen {
                self.screen = screen;
            }
        }
        Ok(())
    }
}

pub fn start() -> Result<()> {
    color_eyre::install()?;

    let mut terminal = ratatui::init();

    let enhanced_keyboard = crossterm::terminal::supports_keyboard_enhancement()?;
    if enhanced_keyboard {
        execute!(
            stdout(),
            PushKeyboardEnhancementFlags(
                KeyboardEnhancementFlags::REPORT_EVENT_TYPES
                    | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
            ),
            EnableFocusChange,
            EnableMouseCapture
        )?;
    }

    let result = App::new(enhanced_keyboard).run(&mut terminal);

    if enhanced_keyboard {
        execute!(
            stdout(),
            DisableMouseCapture,
            DisableFocusChange,
            PopKeyboardEnhancementFlags
        )?;
    }

    ratatui::restore();
    result
}
