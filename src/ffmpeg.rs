use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use thiserror::Error;

use crate::converter::{ConversionProgress, VideoFormat};

#[derive(Error, Debug)]
pub enum FFmpegError {
    #[error("FFmpeg not found on system")]
    NotFound,
    
    #[error("Failed to execute FFmpeg: {0}")]
    ExecutionError(#[from] std::io::Error),
    
    #[error("FFmpeg process failed with status: {0}")]
    ProcessError(i32),
    
    #[error("FFmpeg process terminated by signal")]
    ProcessTerminated,
    
    #[error("Invalid input file")]
    InvalidInput,
}

pub struct FFmpegConverter {
    progress_tx: mpsc::Sender<ConversionProgress>,
}

impl FFmpegConverter {
    pub fn new(progress_tx: mpsc::Sender<ConversionProgress>) -> Self {
        Self { progress_tx }
    }
    
    pub fn check_ffmpeg_available() -> Result<bool, FFmpegError> {
        match Command::new("ffmpeg").arg("-version").output() {
            Ok(_) => Ok(true),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    Ok(false)
                } else {
                    Err(FFmpegError::ExecutionError(e))
                }
            }
        }
    }
    
    fn get_video_duration(source_file: &PathBuf) -> Result<f64, FFmpegError> {
        // Use FFprobe to get video duration
        let output = Command::new("ffprobe")
            .arg("-v").arg("error")
            .arg("-show_entries").arg("format=duration")
            .arg("-of").arg("default=noprint_wrappers=1:nokey=1")
            .arg(source_file)
            .output()
            .map_err(FFmpegError::ExecutionError)?;
        
        if !output.status.success() {
            return Err(FFmpegError::ProcessError(output.status.code().unwrap_or(-1)));
        }
        
        // Parse the duration
        let duration_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        duration_str.parse::<f64>().map_err(|_| FFmpegError::InvalidInput)
    }
    
    pub fn convert(&self, source_file: PathBuf, target_format: VideoFormat, output_file: PathBuf) -> Result<(), FFmpegError> {
        // Verify source file exists
        if !source_file.exists() {
            return Err(FFmpegError::InvalidInput);
        }
        
        // Start conversion in a separate thread
        let progress_tx = self.progress_tx.clone();
        
        thread::spawn(move || {
            // Send initial progress
            Self::send_progress(
                &progress_tx,
                0,
                "Starting FFmpeg conversion...".to_string(),
                &source_file,
                target_format,
                &output_file,
                false,
                false,
                None
            );
            
            // First, get video duration
            let duration_seconds = Self::get_video_duration(&source_file);
            
            // Send analyzing progress
            Self::send_progress(
                &progress_tx,
                0,
                format!("Analyzing video file... Duration: {} seconds", 
                    duration_seconds.unwrap_or_else(|_| 0.0)),
                &source_file,
                target_format,
                &output_file,
                false,
                false,
                None
            );
            
            // Build FFmpeg command with appropriate options based on format
            let mut cmd = Command::new("ffmpeg");
            
            // Add input file
            cmd.arg("-i")
               .arg(&source_file)
               .arg("-y"); // Overwrite output files without asking
               
            // Add format-specific options
            match target_format {
                VideoFormat::MP4 => {
                    // H.264 video with AAC audio - good compatibility
                    cmd.arg("-c:v").arg("libx264")
                       .arg("-preset").arg("medium")
                       .arg("-crf").arg("23")
                       .arg("-c:a").arg("aac")
                       .arg("-b:a").arg("128k");
                },
                VideoFormat::MKV => {
                    // H.264 video with high quality
                    cmd.arg("-c:v").arg("libx264")
                       .arg("-preset").arg("slow")
                       .arg("-crf").arg("18")
                       .arg("-c:a").arg("copy");
                },
                VideoFormat::AVI => {
                    // MPEG-4 video for compatibility
                    cmd.arg("-c:v").arg("mpeg4")
                       .arg("-q:v").arg("6")
                       .arg("-c:a").arg("libmp3lame")
                       .arg("-q:a").arg("4");
                },
                VideoFormat::MOV => {
                    // ProRes for high quality
                    cmd.arg("-c:v").arg("prores_ks")
                       .arg("-profile:v").arg("3")
                       .arg("-c:a").arg("pcm_s16le");
                },
                VideoFormat::WEBM => {
                    // VP9 video with Opus audio - good for web
                    cmd.arg("-c:v").arg("libvpx-vp9")
                       .arg("-crf").arg("30")
                       .arg("-b:v").arg("0")
                       .arg("-c:a").arg("libopus")
                       .arg("-b:a").arg("96k");
                },
            }
            
            // Add progress reporting
            cmd.arg("-progress")
               .arg("pipe:1") // Output progress information to stdout
               .arg(&output_file);
            
            // Configure stdio
            cmd.stdout(Stdio::piped())
               .stderr(Stdio::piped());
            
            // Execute command
            match cmd.spawn() {
                Ok(mut child) => {
                    // Get stdout for progress tracking
                    let stdout = child.stdout.take().unwrap();
                    let reader = BufReader::new(stdout);
                    
                    // Track progress
                    let mut duration_ms: f64 = 0.0;
                    let mut time_ms: f64 = 0.0;
                    
                    // Parse FFmpeg progress output
                    for line in reader.lines() {
                        if let Ok(line) = line {
                            // Parse progress information
                            if line.starts_with("out_time_ms=") {
                                if let Ok(time) = line[12..].parse::<f64>() {
                                    time_ms = time;
                                    
                                    // Calculate progress percentage if we have duration
                                    if duration_ms > 0.0 {
                                        let percent = ((time_ms / duration_ms) * 100.0).min(100.0) as u8;
                                        
                                        Self::send_progress(
                                            &progress_tx,
                                            percent,
                                            format!("Converting video... {}%", percent),
                                            &source_file,
                                            target_format,
                                            &output_file,
                                            false,
                                            false,
                                            None
                                        );
                                    }
                                }
                            } else if line.starts_with("duration=") {
                                if let Ok(time) = line[9..].parse::<f64>() {
                                    duration_ms = time * 1000.0;
                                }
                            } else if line == "progress=end" {
                                // Conversion complete
                                Self::send_progress(
                                    &progress_tx,
                                    100,
                                    "Conversion complete!".to_string(),
                                    &source_file,
                                    target_format,
                                    &output_file,
                                    true,
                                    false,
                                    None
                                );
                                break;
                            }
                        }
                    }
                    
                    // Wait for process to complete
                    match child.wait() {
                        Ok(status) => {
                            if !status.success() {
                                if let Some(code) = status.code() {
                                    Self::send_progress(
                                        &progress_tx,
                                        0,
                                        format!("FFmpeg failed with exit code: {}", code),
                                        &source_file,
                                        target_format,
                                        &output_file,
                                        true,
                                        true,
                                        Some(format!("FFmpeg process failed with status: {}", code))
                                    );
                                } else {
                                    Self::send_progress(
                                        &progress_tx,
                                        0,
                                        "FFmpeg process terminated by signal".to_string(),
                                        &source_file,
                                        target_format,
                                        &output_file,
                                        true,
                                        true,
                                        Some("FFmpeg process terminated by signal".to_string())
                                    );
                                }
                            }
                        },
                        Err(e) => {
                            Self::send_progress(
                                &progress_tx,
                                0,
                                format!("Error waiting for FFmpeg: {}", e),
                                &source_file,
                                target_format,
                                &output_file,
                                true,
                                true,
                                Some(format!("Error waiting for FFmpeg: {}", e))
                            );
                        }
                    }
                },
                Err(e) => {
                    Self::send_progress(
                        &progress_tx,
                        0,
                        format!("Failed to start FFmpeg: {}", e),
                        &source_file,
                        target_format,
                        &output_file,
                        true,
                        true,
                        Some(format!("Failed to start FFmpeg: {}", e))
                    );
                }
            }
        });
        
        Ok(())
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
        });
    }
}