use std::path::PathBuf;
use std::sync::mpsc;

use crate::converter::{ConversionMode, ConversionProgress, VideoConverter, VideoFormat};
use crate::file_browser::FileBrowser;

// Application tabs
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppTab {
    FileBrowser,
    FormatSelection,
    Converting,
    Complete,
    Settings,
    Help,
}

// Application state
pub struct App {
    pub current_tab: AppTab,
    pub file_browser: FileBrowser,
    pub selected_format: Option<VideoFormat>,
    pub selected_format_idx: usize,
    pub should_quit: bool,
    pub show_popup: bool,
    pub conversion_progress: Option<ConversionProgress>,
    pub converter_rx: Option<mpsc::Receiver<ConversionProgress>>,
}

impl App {
    pub fn new() -> Self {
        // Start in the current directory
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        
        Self {
            current_tab: AppTab::FileBrowser,
            file_browser: FileBrowser::new(current_dir),
            selected_format: None,
            selected_format_idx: 0,
            should_quit: false,
            show_popup: false,
            conversion_progress: None,
            converter_rx: None,
        }
    }

    pub fn next_format(&mut self) {
        self.selected_format_idx = (self.selected_format_idx + 1) % 5;
        self.update_selected_format();
    }

    pub fn previous_format(&mut self) {
        if self.selected_format_idx > 0 {
            self.selected_format_idx -= 1;
        } else {
            self.selected_format_idx = 4;
        }
        self.update_selected_format();
    }
    
    fn update_selected_format(&mut self) {
        self.selected_format = Some(match self.selected_format_idx {
            0 => VideoFormat::MP4,
            1 => VideoFormat::MKV,
            2 => VideoFormat::AVI,
            3 => VideoFormat::MOV,
            _ => VideoFormat::WEBM,
        });
    }
    
    pub fn next_tab(&mut self) {
        self.current_tab = match self.current_tab {
            AppTab::FileBrowser => AppTab::FormatSelection,
            AppTab::FormatSelection => AppTab::Settings,
            AppTab::Settings => AppTab::Help,
            AppTab::Help => AppTab::FileBrowser,
            // Don't change tabs during conversion or when complete
            AppTab::Converting => AppTab::Converting,
            AppTab::Complete => AppTab::Complete,
        };
    }
    
    pub fn previous_tab(&mut self) {
        self.current_tab = match self.current_tab {
            AppTab::FileBrowser => AppTab::Help,
            AppTab::FormatSelection => AppTab::FileBrowser,
            AppTab::Settings => AppTab::FormatSelection,
            AppTab::Help => AppTab::Settings,
            // Don't change tabs during conversion or when complete
            AppTab::Converting => AppTab::Converting,
            AppTab::Complete => AppTab::Complete,
        };
    }
    
    pub fn toggle_popup(&mut self) {
        self.show_popup = !self.show_popup;
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
    
    pub fn get_current_format(&self) -> VideoFormat {
        self.selected_format.unwrap_or(VideoFormat::MP4)
    }
    
    pub fn start_conversion(&mut self) {
        if let Some(file_path) = self.file_browser.get_selected_file() {
            if file_path.is_file() {
                let format = self.get_current_format();
                
                // First try to use native FFmpeg library
                let native_available = match crate::native_converter::NativeConverter::check_available() {
                    Ok(available) => available,
                    Err(_) => false
                };
                
                // If native library not available, check for external FFmpeg
                let ffmpeg_available = if !native_available {
                    match crate::ffmpeg::FFmpegConverter::check_ffmpeg_available() {
                        Ok(available) => available,
                        Err(_) => false
                    }
                } else {
                    false // Skip external FFmpeg check if native is available
                };
                
                // Create converter with appropriate mode
                let mode = if native_available {
                    ConversionMode::NativeFFmpeg
                } else if ffmpeg_available {
                    ConversionMode::FFmpeg
                } else {
                    ConversionMode::Simulation
                };
                
                let (converter, rx) = VideoConverter::new(mode);
                self.converter_rx = Some(rx);
                
                // Start conversion
                converter.convert(file_path.clone(), format);
                
                // Switch to converting tab
                self.current_tab = AppTab::Converting;
            }
        }
    }
    
    pub fn check_conversion_progress(&mut self) {
        if let Some(rx) = &self.converter_rx {
            if let Ok(progress) = rx.try_recv() {
                self.conversion_progress = Some(progress.clone());
                
                if progress.is_complete {
                    self.current_tab = AppTab::Complete;
                }
            }
        }
    }
    
    pub fn reset(&mut self) {
        self.current_tab = AppTab::FileBrowser;
        self.conversion_progress = None;
        self.converter_rx = None;
    }
}