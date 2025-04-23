use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, BorderType, Gauge, List, ListItem, Paragraph, Tabs},
    Frame,
};

use crate::app::{App, AppTab, AdvancedSetting};
use crate::converter::VideoFormat;

pub fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let size = f.size();
    
    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Length(3),  // Tabs
            Constraint::Min(0),     // Content
            Constraint::Length(3),  // Status bar
        ].as_ref())
        .split(size);

    // Title
    render_title(f, chunks[0]);
    
    // Tabs
    render_tabs(f, app, chunks[1]);
    
    // Content based on selected tab
    match app.current_tab {
        AppTab::FileBrowser => render_file_browser(f, app, chunks[2]),
        AppTab::FormatSelection => render_format_selection(f, app, chunks[2]),
        AppTab::Converting => render_converting(f, app, chunks[2]),
        AppTab::Complete => render_complete(f, app, chunks[2]),
        AppTab::Settings => render_settings(f, app, chunks[2]),
        AppTab::Help => render_help(f, chunks[2]),
    }
    
    // Status bar
    render_status_bar(f, app, chunks[3]);
    
    // Render popup if active
    if app.show_popup {
        render_popup(f, app, size);
    }
}

fn render_title<B: Backend>(f: &mut Frame<B>, area: Rect) {
    // Create a block for the header
    let header_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(Color::Cyan));
    
    // Create a solid background for contrast
    let background = Block::default()
        .style(Style::default().bg(Color::Black));
    
    f.render_widget(background, area);
    
    // Calculate the inner area before rendering the block
    let inner_area = header_block.inner(area);
    
    // Now render the block
    f.render_widget(header_block, area);
    
    // Simple text header
    let title_text = "VIDEO FILE CONVERTER";
    
    // Create a centered title with bold styling
    let title = Paragraph::new(title_text)
        .style(Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE));
    
    f.render_widget(title, inner_area);
    
    // Check which conversion tools are available
    let (status_color, native_status) = match crate::native_converter::NativeConverter::check_available() {
        Ok(true) => (Color::Green, "Native Rust Converter: ✅ Ready"),
        _ => match crate::ffmpeg::FFmpegConverter::check_ffmpeg_available() {
            Ok(true) => (Color::Green, "External FFmpeg: ✅ Ready"),
            _ => (Color::Red, "Converters: ❌ Not detected (using simulation)"),
        },
    };
    
    // Add version info with status color
    let version_text = format!("v1.0 | {}", native_status);
    let version_area = Rect {
        x: inner_area.x + 2,
        y: inner_area.y + 2,
        width: inner_area.width - 4,
        height: 1,
    };
    
    let version_info = Paragraph::new(version_text)
        .style(Style::default().fg(status_color))
        .alignment(Alignment::Center);
    
    // Only render version info if there's enough space
    if inner_area.height > 3 {
        f.render_widget(version_info, version_area);
    }
}

fn render_tabs<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let tab_titles = vec!["File Browser", "Format Selection", "Settings", "Help"];
    let tabs = Tabs::new(
        tab_titles
            .iter()
            .map(|t| Spans::from(Span::styled(*t, Style::default().fg(Color::White))))
            .collect(),
    )
    .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded))
    .select(match app.current_tab {
        AppTab::FileBrowser => 0,
        AppTab::FormatSelection => 1,
        AppTab::Settings => 2,
        AppTab::Help => 3,
        // During conversion or when complete, keep the format selection tab highlighted
        AppTab::Converting => 1,
        AppTab::Complete => 1,
    })
    .style(Style::default().fg(Color::White))
    .highlight_style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    );
    
    f.render_widget(tabs, area);
}

fn render_file_browser<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let files = app.file_browser.get_files();
    let selected_idx = app.file_browser.get_selected_idx();
    
    // Create layout for file browser
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Current directory
            Constraint::Min(0),     // File list
        ].as_ref())
        .split(area);
    
    // Current directory display
    let current_dir = app.file_browser.get_current_dir().to_string_lossy();
    let dir_display = Paragraph::new(Spans::from(vec![
        Span::styled("📂 ", Style::default().fg(Color::Yellow)),
        Span::styled(current_dir.to_string(), Style::default().fg(Color::White)),
    ]))
    .style(Style::default().fg(Color::White));
    
    f.render_widget(dir_display, chunks[0]);
    
    // File list
    let items: Vec<ListItem> = files
        .iter()
        .map(|path| {
            let display_text = app.file_browser.format_path_for_display(path);
            let style = if path.is_dir() {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::White)
            };
            
            ListItem::new(Spans::from(display_text)).style(style)
        })
        .collect();

    let files_list = List::new(items)
        .block(
            Block::default()
                .title(" Files ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Blue))
        )
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .highlight_symbol("➤ ");

    let mut state = ratatui::widgets::ListState::default();
    state.select(Some(selected_idx));
    
    f.render_stateful_widget(files_list, chunks[1], &mut state);
}

fn render_format_selection<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let formats = vec![
        VideoFormat::MP4,
        VideoFormat::MKV,
        VideoFormat::AVI,
        VideoFormat::MOV,
        VideoFormat::WEBM,
    ];
    
    // Split the area into two parts: format list and format details
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(70),
        ].as_ref())
        .split(area);
    
    // Format list
    let items: Vec<ListItem> = formats
        .iter()
        .map(|format| {
            let style = if *format == app.get_current_format() {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            
            let format_name = format.as_str();
            ListItem::new(Spans::from(format_name)).style(style)
        })
        .collect();

    let formats_list = List::new(items)
        .block(
            Block::default()
                .title(" Output Formats ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Blue))
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol(" ");

    let mut state = ratatui::widgets::ListState::default();
    state.select(Some(app.selected_format_idx));
    
    f.render_stateful_widget(formats_list, chunks[0], &mut state);
    
    // Format details
    let current_format = app.get_current_format();
    let format_details = vec![
        Spans::from(vec![
            Span::styled("Format: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(current_format.as_str(), Style::default().fg(Color::White)),
        ]),
        Spans::from(""),
        Spans::from(vec![
            Span::styled("Description: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(current_format.description(), Style::default().fg(Color::White)),
        ]),
        Spans::from(""),
        Spans::from(vec![
            Span::styled("Common Use Cases:", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        ]),
        Spans::from(match current_format {
            VideoFormat::MP4 => " Streaming videos online\n Sharing on social media\n Compatible with most devices",
            VideoFormat::MKV => " High-quality video storage\n Multiple audio tracks\n Subtitle support",
            VideoFormat::AVI => " Legacy systems\n Older media players\n Simple editing workflows",
            VideoFormat::MOV => " Apple devices\n Professional video editing\n High-quality recording",
            VideoFormat::WEBM => " Web embedding\n HTML5 video\n Efficient streaming",
        }),
    ];

    let details_widget = Paragraph::new(format_details)
        .block(
            Block::default()
                .title(" Format Details ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Blue))
        )
        .alignment(Alignment::Left);

    f.render_widget(details_widget, chunks[1]);
}

fn render_converting<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    if let Some(progress) = &app.conversion_progress {
        // Determine which conversion tool is being used
        let (tool_color, conversion_tool) = match crate::native_converter::NativeConverter::check_available() {
            Ok(true) => (Color::Green, "Native Rust FFmpeg"),
            _ => match crate::ffmpeg::FFmpegConverter::check_ffmpeg_available() {
                Ok(true) => (Color::Green, "External FFmpeg"),
                _ => (Color::Yellow, "Simulation Mode"),
            },
        };
        
        // Create layout for conversion display
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Source file
                Constraint::Length(3),  // Target format
                Constraint::Length(3),  // Output file
                Constraint::Length(3),  // Conversion method
                Constraint::Length(3),  // Current step
                Constraint::Length(3),  // Progress bar
                Constraint::Min(0),     // Spacer
            ].as_ref())
            .split(area);
        
        // Source file
        let source_file = Paragraph::new(Spans::from(vec![
            Span::styled("Source File: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(
                progress.source_file.file_name().unwrap_or_default().to_string_lossy().to_string(), 
                Style::default().fg(Color::White)
            ),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Blue))
        );
        
        // Target format
        let target_format = Paragraph::new(Spans::from(vec![
            Span::styled("Target Format: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(
                progress.target_format.as_str(), 
                Style::default().fg(Color::White)
            ),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Blue))
        );
        
        // Output file
        let output_file = Paragraph::new(Spans::from(vec![
            Span::styled("Output File: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(
                progress.output_file.file_name().unwrap_or_default().to_string_lossy().to_string(), 
                Style::default().fg(Color::White)
            ),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Blue))
        );
        
        // Conversion method
        let conversion_method = Paragraph::new(Spans::from(vec![
            Span::styled("Conversion Method: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(conversion_tool, Style::default().fg(tool_color)),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Blue))
        );
        
        // Current step
        let current_step = Paragraph::new(Spans::from(vec![
            Span::styled("Current Step: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(
                &progress.current_step, 
                Style::default().fg(Color::White)
            ),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Blue))
        );
        
        // Progress bar
        let progress_gauge = Gauge::default()
            .block(
                Block::default()
                    .title(" Progress ")
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Blue))
            )
            .gauge_style(Style::default().fg(Color::Cyan))
            .ratio(f64::from(progress.percent) / 100.0)
            .label(format!("{}%", progress.percent));
        
        f.render_widget(source_file, chunks[0]);
        f.render_widget(target_format, chunks[1]);
        f.render_widget(output_file, chunks[2]);
        f.render_widget(conversion_method, chunks[3]);
        f.render_widget(current_step, chunks[4]);
        f.render_widget(progress_gauge, chunks[5]);
    }
}

fn render_complete<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    if let Some(progress) = &app.conversion_progress {
        // Determine which conversion tool was used
        let (tool_color, conversion_tool) = match crate::native_converter::NativeConverter::check_available() {
            Ok(true) => (Color::Green, "Native Rust FFmpeg"),
            _ => match crate::ffmpeg::FFmpegConverter::check_ffmpeg_available() {
                Ok(true) => (Color::Green, "External FFmpeg"),
                _ => (Color::Yellow, "Simulation Mode"),
            },
        };
        
        let text = vec![
            Spans::from(vec![
                Span::styled("✅ Conversion Complete!", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            ]),
            Spans::from(""),
            Spans::from(vec![
                Span::styled("Source File: ", Style::default().fg(Color::Green)),
                Span::styled(
                    progress.source_file.file_name().unwrap_or_default().to_string_lossy().to_string(), 
                    Style::default().fg(Color::White)
                ),
            ]),
            Spans::from(vec![
                Span::styled("Output Format: ", Style::default().fg(Color::Green)),
                Span::styled(
                    progress.target_format.as_str(), 
                    Style::default().fg(Color::White)
                ),
            ]),
            Spans::from(vec![
                Span::styled("Output File: ", Style::default().fg(Color::Green)),
                Span::styled(
                    progress.output_file.file_name().unwrap_or_default().to_string_lossy().to_string(), 
                    Style::default().fg(Color::White)
                ),
            ]),
            Spans::from(vec![
                Span::styled("Conversion Method: ", Style::default().fg(Color::Green)),
                Span::styled(
                    conversion_tool, 
                    Style::default().fg(tool_color)
                ),
            ]),
            Spans::from(""),
            Spans::from(vec![
                Span::styled("Press 'n' to convert another file or 'q' to quit", Style::default().fg(Color::Yellow)),
            ]),
        ];

        let completion_widget = Paragraph::new(text)
            .block(
                Block::default()
                    .title(" Conversion Result ")
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Blue))
            )
            .alignment(Alignment::Center);
        
        f.render_widget(completion_widget, area);
    }
}

fn render_settings<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    // Determine which conversion tool is available
    let conversion_tool = match crate::native_converter::NativeConverter::check_available() {
        Ok(true) => "Native Rust FFmpeg",
        _ => match crate::ffmpeg::FFmpegConverter::check_ffmpeg_available() {
            Ok(true) => "External FFmpeg",
            _ => "Simulation Mode",
        },
    };
    
    // Create layout for settings sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // Conversion tool
            Constraint::Length(1),  // Spacer
            Constraint::Length(10), // Advanced video settings
            Constraint::Min(0),     // Future settings
        ].as_ref())
        .split(area);
    
    // Conversion tool section
    let tool_block = Block::default()
        .title(" Conversion Tool ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    
    // Calculate inner area before rendering the block
    let tool_inner = tool_block.inner(chunks[0]);
    
    // Render the block first
    f.render_widget(tool_block, chunks[0]);
    
    // Then render the text in the inner area
    let tool_text = Paragraph::new(conversion_tool)
        .style(Style::default().fg(Color::Green))
        .alignment(Alignment::Center);
    
    f.render_widget(tool_text, tool_inner);
    
    // Advanced video settings section
    let settings_block = Block::default()
        .title(" Advanced Video Settings ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    
    // Calculate inner area before rendering the block
    let settings_area = settings_block.inner(chunks[2]);
    
    // Render the block
    f.render_widget(settings_block, chunks[2]);
    let settings_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1),  // Resolution
            Constraint::Length(1),  // Bitrate
            Constraint::Length(1),  // Frame Rate
            Constraint::Length(1),  // Spacer
            Constraint::Length(1),  // Instructions
        ].as_ref())
        .split(settings_area);
    
    // Resolution setting
    let resolution_text = format!("Resolution: {}", app.video_settings.resolution.as_str());
    let resolution_style = if app.selected_setting == AdvancedSetting::Resolution {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    let resolution_para = Paragraph::new(resolution_text).style(resolution_style);
    f.render_widget(resolution_para, settings_layout[0]);
    
    // Bitrate setting
    let bitrate_text = format!("Bitrate: {}", app.video_settings.bitrate.as_str());
    let bitrate_style = if app.selected_setting == AdvancedSetting::Bitrate {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    let bitrate_para = Paragraph::new(bitrate_text).style(bitrate_style);
    f.render_widget(bitrate_para, settings_layout[1]);
    
    // Frame rate setting
    let framerate_text = format!("Frame Rate: {}", app.video_settings.frame_rate.as_str());
    let framerate_style = if app.selected_setting == AdvancedSetting::FrameRate {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    let framerate_para = Paragraph::new(framerate_text).style(framerate_style);
    f.render_widget(framerate_para, settings_layout[2]);
    
    // Instructions
    let instructions = Paragraph::new("↑/↓: Select setting | ←/→: Change value")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    f.render_widget(instructions, settings_layout[4]);
}

fn render_help<B: Backend>(f: &mut Frame<B>, area: Rect) {
    let help_text = vec![
        Spans::from(vec![
            Span::styled("Keyboard Controls", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Spans::from(""),
        Spans::from(vec![
            Span::styled("↑/↓: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled("Navigate through files/formats", Style::default().fg(Color::White)),
        ]),
        Spans::from(vec![
            Span::styled("Enter: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled("Open directory / Select file / Start conversion", Style::default().fg(Color::White)),
        ]),
        Spans::from(vec![
            Span::styled("←/→ or Tab: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled("Switch tabs", Style::default().fg(Color::White)),
        ]),
        Spans::from(vec![
            Span::styled("n: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled("Start new conversion (after completion)", Style::default().fg(Color::White)),
        ]),
        Spans::from(vec![
            Span::styled("p: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled("Show/hide popup", Style::default().fg(Color::White)),
        ]),
        Spans::from(vec![
            Span::styled("q: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled("Quit application", Style::default().fg(Color::White)),
        ]),
        Spans::from(""),
        Spans::from(vec![
            Span::styled("About", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Spans::from(""),
        Spans::from("This application allows you to convert video files to different formats."),
        Spans::from("Browse for a file, select a format, and press Enter to start the conversion."),
    ];

    let help_widget = Paragraph::new(help_text)
        .block(
            Block::default()
                .title(" Help & Information ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Blue))
        )
        .alignment(Alignment::Left);

    f.render_widget(help_widget, area);
}

fn render_status_bar<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let status_text = match app.current_tab {
        AppTab::FileBrowser => {
            if let Some(path) = app.file_browser.get_selected_file() {
                if path.is_dir() {
                    "Press Enter to open directory | Tab: Switch tabs | q: Quit".to_string()
                } else {
                    "Press Enter to select file | Tab: Switch tabs | q: Quit".to_string()
                }
            } else {
                "No files found | Tab: Switch tabs | q: Quit".to_string()
            }
        },
        AppTab::FormatSelection => format!("Selected Format: {} | Press Enter to convert | Tab: Switch tabs | q: Quit", app.get_current_format().as_str()),
        AppTab::Converting => "Converting... Please wait | q: Quit".to_string(),
        AppTab::Complete => "Conversion complete! Press 'n' for new conversion | q: Quit".to_string(),
        AppTab::Settings => "Settings | Tab: Switch tabs | q: Quit".to_string(),
        AppTab::Help => "Help & Information | Tab: Switch tabs | q: Quit".to_string(),
    };
    
    let status_bar = Paragraph::new(Spans::from(vec![
        Span::styled(" ● ", Style::default().fg(Color::Green)),
        Span::styled(&status_text, Style::default().fg(Color::White)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::DarkGray))
    );
    
    f.render_widget(status_bar, area);
}

fn render_popup<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let popup_area = centered_rect(60, 20, area);
    
    // Clear the area
    f.render_widget(
        Block::default()
            .style(Style::default().bg(Color::Black)),
        popup_area,
    );
    
    // Determine which conversion tool is available
    let (tool_color, conversion_tool) = match crate::native_converter::NativeConverter::check_available() {
        Ok(true) => (Color::Green, "Native Rust FFmpeg"),
        _ => match crate::ffmpeg::FFmpegConverter::check_ffmpeg_available() {
            Ok(true) => (Color::Green, "External FFmpeg"),
            _ => (Color::Yellow, "Simulation Mode"),
        },
    };
    
    let current_format = app.get_current_format();
    let popup_text = if let Some(file_path) = app.file_browser.get_selected_file() {
        if file_path.is_file() {
            let file_name = file_path.file_name().unwrap_or_default().to_string_lossy();
            vec![
                Spans::from(vec![
                    Span::styled("Ready to Convert", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                ]),
                Spans::from(""),
                Spans::from(vec![
                    Span::styled("File: ", Style::default().fg(Color::Green)),
                    Span::styled(file_name.to_string(), Style::default().fg(Color::White)),
                ]),
                Spans::from(vec![
                    Span::styled("Format: ", Style::default().fg(Color::Green)),
                    Span::styled(current_format.as_str(), Style::default().fg(Color::White)),
                ]),
                Spans::from(vec![
                    Span::styled("Using: ", Style::default().fg(Color::Green)),
                    Span::styled(conversion_tool, Style::default().fg(tool_color)),
                ]),
                Spans::from(""),
                Spans::from(vec![
                    Span::styled("Video Settings: ", Style::default().fg(Color::Green)),
                ]),
                Spans::from(vec![
                    Span::styled("  Resolution: ", Style::default().fg(Color::Cyan)),
                    Span::styled(app.video_settings.resolution.as_str(), Style::default().fg(Color::White)),
                ]),
                Spans::from(vec![
                    Span::styled("  Bitrate: ", Style::default().fg(Color::Cyan)),
                    Span::styled(app.video_settings.bitrate.as_str(), Style::default().fg(Color::White)),
                ]),
                Spans::from(vec![
                    Span::styled("  Frame Rate: ", Style::default().fg(Color::Cyan)),
                    Span::styled(app.video_settings.frame_rate.as_str(), Style::default().fg(Color::White)),
                ]),
                Spans::from(""),
                Spans::from("Press Enter to start conversion or Esc to cancel."),
            ]
        } else {
            vec![
                Spans::from(vec![
                    Span::styled("Cannot Convert Directory", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                ]),
                Spans::from(""),
                Spans::from("Please select a video file to convert."),
                Spans::from(""),
                Spans::from("Press Esc to close this popup."),
            ]
        }
    } else {
        vec![
            Spans::from(vec![
                Span::styled("No File Selected", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            ]),
            Spans::from(""),
            Spans::from("Please select a video file to convert."),
            Spans::from(""),
            Spans::from("Press Esc to close this popup."),
        ]
    };
    
    let popup = Paragraph::new(popup_text)
        .block(
            Block::default()
                .title(" Conversion Confirmation ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(Style::default().fg(Color::Cyan))
        )
        .alignment(Alignment::Center);
    
    f.render_widget(popup, popup_area);
}

// Helper function to create a centered rect using a percentage of the available rect
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ].as_ref())
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ].as_ref())
        .split(popup_layout[1])[1]
}