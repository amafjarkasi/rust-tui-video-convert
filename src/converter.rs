use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Resolution {
    Original,
    HD720p,
    HD1080p,
    UHD4K,
}

impl Resolution {
    pub fn as_str(&self) -> &'static str {
        match self {
            Resolution::Original => "Original",
            Resolution::HD720p => "720p",
            Resolution::HD1080p => "1080p",
            Resolution::UHD4K => "4K",
        }
    }
    
    pub fn dimensions(&self) -> Option<(u32, u32)> {
        match self {
            Resolution::Original => None,
            Resolution::HD720p => Some((1280, 720)),
            Resolution::HD1080p => Some((1920, 1080)),
            Resolution::UHD4K => Some((3840, 2160)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Bitrate {
    Auto,
    Low,
    Medium,
    High,
}

impl Bitrate {
    pub fn as_str(&self) -> &'static str {
        match self {
            Bitrate::Auto => "Auto",
            Bitrate::Low => "Low",
            Bitrate::Medium => "Medium",
            Bitrate::High => "High",
        }
    }
    
    pub fn value_kbps(&self, resolution: &Resolution) -> u32 {
        match (self, resolution) {
            (Bitrate::Auto, _) => 0, // Let the converter decide
            (Bitrate::Low, Resolution::HD720p) => 1500,
            (Bitrate::Medium, Resolution::HD720p) => 2500,
            (Bitrate::High, Resolution::HD720p) => 4000,
            (Bitrate::Low, Resolution::HD1080p) => 3000,
            (Bitrate::Medium, Resolution::HD1080p) => 6000,
            (Bitrate::High, Resolution::HD1080p) => 8000,
            (Bitrate::Low, Resolution::UHD4K) => 8000,
            (Bitrate::Medium, Resolution::UHD4K) => 12000,
            (Bitrate::High, Resolution::UHD4K) => 18000,
            _ => 6000, // Default medium quality for other combinations
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FrameRate {
    Original,
    FPS24,
    FPS30,
    FPS60,
}

impl FrameRate {
    pub fn as_str(&self) -> &'static str {
        match self {
            FrameRate::Original => "Original",
            FrameRate::FPS24 => "24 fps",
            FrameRate::FPS30 => "30 fps",
            FrameRate::FPS60 => "60 fps",
        }
    }
    
    pub fn value(&self) -> Option<u32> {
        match self {
            FrameRate::Original => None,
            FrameRate::FPS24 => Some(24),
            FrameRate::FPS30 => Some(30),
            FrameRate::FPS60 => Some(60),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct VideoSettings {
    pub resolution: Resolution,
    pub bitrate: Bitrate,
    pub frame_rate: FrameRate,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VideoFormat {
    MP4,
    MKV,
    AVI,
    MOV,
    WEBM,
}

impl VideoFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            VideoFormat::MP4 => "MP4",
            VideoFormat::MKV => "MKV",
            VideoFormat::AVI => "AVI",
            VideoFormat::MOV => "MOV",
            VideoFormat::WEBM => "WEBM",
        }
    }
    
    pub fn extension(&self) -> &'static str {
        match self {
            VideoFormat::MP4 => "mp4",
            VideoFormat::MKV => "mkv",
            VideoFormat::AVI => "avi",
            VideoFormat::MOV => "mov",
            VideoFormat::WEBM => "webm",
        }
    }
    
    pub fn description(&self) -> &'static str {
        match self {
            VideoFormat::MP4 => "MPEG-4 Part 14 - Widely supported format with good compression",
            VideoFormat::MKV => "Matroska Video - Container format that can hold many codecs",
            VideoFormat::AVI => "Audio Video Interleave - Microsoft's container format",
            VideoFormat::MOV => "QuickTime File Format - Apple's container format",
            VideoFormat::WEBM => "WebM - Open, royalty-free format designed for the web",
        }
    }
    
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "mp4" => Some(VideoFormat::MP4),
            "mkv" => Some(VideoFormat::MKV),
            "avi" => Some(VideoFormat::AVI),
            "mov" => Some(VideoFormat::MOV),
            "webm" => Some(VideoFormat::WEBM),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct ConversionProgress {
    pub percent: u8,
    pub current_step: String,
    pub source_file: PathBuf,
    pub target_format: VideoFormat,
    pub output_file: PathBuf,
    pub is_complete: bool,
    pub has_error: bool,
    pub error_message: Option<String>,
    pub video_settings: Option<VideoSettings>,
}

pub enum ConversionMode {
    Simulation,
    FFmpeg,
    NativeFFmpeg,
}

pub struct VideoConverter {
    progress_tx: mpsc::Sender<ConversionProgress>,
    mode: ConversionMode,
}

impl VideoConverter {
    pub fn new(mode: ConversionMode) -> (Self, mpsc::Receiver<ConversionProgress>) {
        let (progress_tx, progress_rx) = mpsc::channel();
        (Self { progress_tx, mode }, progress_rx)
    }

    pub fn convert(&self, source_file: PathBuf, target_format: VideoFormat) {
        let progress_tx = self.progress_tx.clone();
        
        // Create output file path
        let output_file = Self::generate_output_path(&source_file, target_format);
        
        // Default video settings
        let default_settings = VideoSettings {
            resolution: Resolution::Original,
            bitrate: Bitrate::Auto,
            frame_rate: FrameRate::Original,
        };
        
        // Send initial progress notification
        Self::send_progress(
            &progress_tx, 
            0, 
            "Initializing conversion...".to_string(),
            &source_file,
            target_format,
            &output_file,
            false,
            false,
            None,
            Some(default_settings)
        );
        
        match self.mode {
            ConversionMode::Simulation => {
                self.simulate_conversion(source_file, target_format, output_file)
            },
            
            ConversionMode::NativeFFmpeg => {
                // Use native FFmpeg library
                let native = crate::native_converter::NativeConverter::new(self.progress_tx.clone());
                if let Err(e) = native.convert(source_file.clone(), target_format, output_file.clone()) {
                    // Handle error
                    Self::send_progress(
                        &progress_tx, 
                        0, 
                        format!("Native FFmpeg error: {}, falling back to simulation", e),
                        &source_file,
                        target_format,
                        &output_file,
                        false,
                        true,
                        Some(format!("Native FFmpeg error: {}", e)),
                        None
                    );
                    // Fall back to simulation
                    self.simulate_conversion(source_file, target_format, output_file);
                }
            },
            
            ConversionMode::FFmpeg => {
                // Check if FFmpeg is available
                if let Ok(available) = crate::ffmpeg::FFmpegConverter::check_ffmpeg_available() {
                    if available {
                        // Use FFmpeg for conversion
                        let ffmpeg = crate::ffmpeg::FFmpegConverter::new(self.progress_tx.clone());
                        if let Err(e) = ffmpeg.convert(source_file.clone(), target_format, output_file.clone()) {
                            // Handle error
                            Self::send_progress(
                                &progress_tx, 
                                0, 
                                format!("FFmpeg error: {}, falling back to simulation", e),
                                &source_file,
                                target_format,
                                &output_file,
                                false,
                                true,
                                Some(format!("FFmpeg error: {}", e)),
                                None
                            );
                            // Fall back to simulation
                            self.simulate_conversion(source_file, target_format, output_file);
                        }
                    } else {
                        // FFmpeg not available, fall back to simulation
                        Self::send_progress(
                            &progress_tx, 
                            0, 
                            "FFmpeg not found, using simulation mode".to_string(),
                            &source_file,
                            target_format,
                            &output_file,
                            false,
                            false,
                            None,
                            None
                        );
                        self.simulate_conversion(source_file, target_format, output_file);
                    }
                } else {
                    // Error checking FFmpeg, fall back to simulation
                    Self::send_progress(
                        &progress_tx, 
                        0,
                        "Error checking FFmpeg availability, using simulation mode".to_string(), 
                        &source_file,
                        target_format,
                        &output_file,
                        false,
                        false,
                        None,
                        Some(default_settings)
                    );
                    self.simulate_conversion(source_file, target_format, output_file);
                }
            }
        }
    }
    
    fn simulate_conversion(&self, source_file: PathBuf, target_format: VideoFormat, output_file: PathBuf) {
        let progress_tx = self.progress_tx.clone();
        
        // Spawn a thread to handle the conversion simulation
        thread::spawn(move || {
            // This is a simulation of video conversion
            
            // Step 1: Analyzing video
            Self::send_progress(
                &progress_tx, 
                0, 
                "Analyzing video file...".to_string(),
                &source_file,
                target_format,
                &output_file,
                false,
                false,
                None,
                None
            );
            thread::sleep(Duration::from_millis(500));
            
            // Step 2: Extracting audio
            Self::send_progress(
                &progress_tx, 
                10, 
                "Extracting audio stream...".to_string(),
                &source_file,
                target_format,
                &output_file,
                false,
                false,
                None,
                None
            );
            thread::sleep(Duration::from_millis(1000));
            
            // Step 3: Processing video
            for i in 20..=80 {
                Self::send_progress(
                    &progress_tx, 
                    i, 
                    format!("Converting video frame {}/100...", i),
                    &source_file,
                    target_format,
                    &output_file,
                    false,
                    false,
                    None,
                    None
                );
                thread::sleep(Duration::from_millis(100));
            }
            
            // Step 4: Muxing streams
            Self::send_progress(
                &progress_tx, 
                90, 
                "Muxing audio and video streams...".to_string(),
                &source_file,
                target_format,
                &output_file,
                false,
                false,
                None,
                None
            );
            thread::sleep(Duration::from_millis(500));
            
            // Step 5: Finalizing
            Self::send_progress(
                &progress_tx, 
                100, 
                "Finalizing output file...".to_string(),
                &source_file,
                target_format,
                &output_file,
                false,
                false,
                None,
                None
            );
            thread::sleep(Duration::from_millis(300));
            
            // Complete
            Self::send_progress(
                &progress_tx, 
                100, 
                "Conversion complete!".to_string(),
                &source_file,
                target_format,
                &output_file,
                true,
                false,
                None,
                None
            );
        });
    }
    
    fn send_progress(
        tx: &mpsc::Sender<ConversionProgress>,
        percent: u8,
        step: String,
        source_file: &PathBuf,
        target_format: VideoFormat,
        output_file: &PathBuf,
        is_complete: bool,
        has_error: bool,
        error_message: Option<String>,
        video_settings: Option<VideoSettings>,
    ) {
        let _ = tx.send(ConversionProgress {
            percent,
            current_step: step,
            source_file: source_file.clone(),
            target_format,
            output_file: output_file.clone(),
            is_complete,
            has_error,
            error_message,
            video_settings,
        });
    }
    
    fn generate_output_path(source_file: &PathBuf, target_format: VideoFormat) -> PathBuf {
        let parent = source_file.parent().unwrap_or_else(|| Path::new(""));
        let stem = source_file.file_stem().unwrap_or_default();
        
        let mut output_path = parent.to_path_buf();
        output_path.push(format!("{}.{}", stem.to_string_lossy(), target_format.extension()));
        
        output_path
    }
}