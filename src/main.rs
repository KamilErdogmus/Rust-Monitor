mod app;
mod theme;
mod ui;

use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::DefaultTerminal;

use app::{App, InputMode};

fn main() -> io::Result<()> {
    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal) -> io::Result<()> {
    let mut app = App::new();
    let tick_rate = Duration::from_millis(500);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|frame| ui::draw(frame, &mut app))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                if app.show_help {
                    app.toggle_help();
                    continue;
                }

                if app.kill_confirm.is_some() {
                    match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') => app.confirm_kill(),
                        _ => app.cancel_kill(),
                    }
                    continue;
                }

                if app.input_mode == InputMode::Search {
                    match key.code {
                        KeyCode::Esc => app.exit_search(),
                        KeyCode::Enter => {
                            app.input_mode = InputMode::Normal;
                        }
                        KeyCode::Backspace => app.search_pop(),
                        KeyCode::Char(c) => app.search_push(c),
                        _ => {}
                    }
                    continue;
                }

                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Tab => app.next_tab(),
                    KeyCode::BackTab => app.prev_tab(),
                    KeyCode::Down | KeyCode::Char('j') => app.scroll_down(),
                    KeyCode::Up | KeyCode::Char('k') => app.scroll_up(),
                    KeyCode::PageDown => app.page_down(),
                    KeyCode::PageUp => app.page_up(),
                    KeyCode::Home => app.scroll_to_top(),
                    KeyCode::End => app.scroll_to_bottom(),
                    KeyCode::Char('s') => app.toggle_sort(),
                    KeyCode::Char('t') => app.toggle_theme(),
                    KeyCode::Char('?') => app.toggle_help(),
                    KeyCode::Char('/') => app.enter_search(),
                    KeyCode::Char('x') => app.request_kill(),
                    KeyCode::Char('1') => app.active_tab = app::Tab::Overview,
                    KeyCode::Char('2') => app.active_tab = app::Tab::Processes,
                    KeyCode::Char('3') => app.active_tab = app::Tab::SystemInfo,
                    KeyCode::Char('4') => app.active_tab = app::Tab::NetworkDetail,
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.tick();
            last_tick = Instant::now();
        }
    }
}
