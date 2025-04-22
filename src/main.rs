/*
  _____           _     _______ _    _ _____   __      ___     _                _____                          _            
 |  __ \         | |   |__   __| |  | |_   _|  \ \    / (_)   | |              / ____|                        | |           
 | |__) |   _ ___| |_     | |  | |  | | | |     \ \  / / _  __| | ___  ___    | |     ___  _ ____   _____ _ __| |_ ___ _ __ 
 |  _  / | | / __| __|    | |  | |  | | | |      \ \/ / | |/ _` |/ _ \/ _ \   | |    / _ \| '_ \ \ / / _ \ '__| __/ _ \ '__|
 | | \ \ |_| \__ \ |_     | |  | |__| |_| |_      \  /  | | (_| |  __/ (_) |  | |___| (_) | | | \ V /  __/ |  | ||  __/ |   
 |_|  \_\__,_|___/\__|    |_|   \____/|_____|      \/   |_|\__,_|\___|\___/    \_____\___/|_| |_|\_/ \___|_|   \__\___|_|   
                                                                                                                            
*/

mod app;
mod converter;
mod ffmpeg;
mod file_browser;
mod ui;
mod native_converter;

use std::{io, time::Duration};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

use app::{App, AppTab};
use ui::ui;

fn main() -> Result<(), io::Error> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let app = App::new();
    let res = run_app(&mut terminal, app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {:?}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        // Draw UI
        terminal.draw(|f| ui(f, &app))?;

        // Check for conversion progress
        app.check_conversion_progress();

        // Handle events
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    // Quit application
                    KeyCode::Char('q') => {
                        app.quit();
                    },
                    
                    // Toggle popup
                    KeyCode::Char('p') => {
                        app.toggle_popup();
                    },
                    
                    // New conversion after completion
                    KeyCode::Char('n') => {
                        if app.current_tab == AppTab::Complete {
                            app.reset();
                        }
                    },
                    
                    // Navigation
                    KeyCode::Down => {
                        match app.current_tab {
                            AppTab::FileBrowser => app.file_browser.next(),
                            AppTab::FormatSelection => app.next_format(),
                            _ => {}
                        }
                    },
                    KeyCode::Up => {
                        match app.current_tab {
                            AppTab::FileBrowser => app.file_browser.previous(),
                            AppTab::FormatSelection => app.previous_format(),
                            _ => {}
                        }
                    },
                    
                    // Tab navigation
                    KeyCode::Right | KeyCode::Tab => {
                        app.next_tab();
                    },
                    KeyCode::Left => {
                        app.previous_tab();
                    },
                    
                    // Selection / Action
                    KeyCode::Enter => {
                        match app.current_tab {
                            AppTab::FileBrowser => {
                                // If selected item is a directory, enter it
                                if !app.file_browser.enter_directory() {
                                    // If it's a file, move to format selection
                                    if app.file_browser.is_selected_file() {
                                        app.current_tab = AppTab::FormatSelection;
                                    }
                                }
                            },
                            AppTab::FormatSelection => {
                                // Start conversion
                                app.start_conversion();
                            },
                            _ => {}
                        }
                    },
                    
                    // Close popup with Escape
                    KeyCode::Esc => {
                        if app.show_popup {
                            app.show_popup = false;
                        }
                    },
                    
                    _ => {}
                }
            }
        }

        // Check if we should exit
        if app.should_quit {
            break;
        }
    }

    Ok(())
}