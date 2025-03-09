mod browser;

use crate::data::Activity;
use crate::db::Database;
use chrono::Local;
use rusqlite::Result;
use std::collections::HashMap;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time;
use sysinfo::{System, Process};
use self::browser::BrowserMonitor;
#[cfg(target_os = "macos")]
use {
    cocoa::base::{id, nil},
    cocoa::foundation::{NSString},
    objc::{msg_send, sel, sel_impl, class},
};

pub struct AppMonitor {
    db: Arc<Mutex<Database>>,
    current_app: String,
    current_browser_url: Option<String>,
    last_switch_time: Instant,
    last_save_time: Instant,
    is_running: Arc<Mutex<bool>>,
    app_durations: HashMap<String, Duration>,
    pub sampling_interval: Duration,
}

impl AppMonitor {
    pub fn new(db: Database) -> Self {
        Self {
            db: Arc::new(Mutex::new(db)),
            current_app: String::new(),
            current_browser_url: None,
            last_switch_time: Instant::now(),
            last_save_time: Instant::now(),
            is_running: Arc::new(Mutex::new(false)),
            app_durations: HashMap::new(),
            sampling_interval: Duration::from_secs(5), // Check every 5 seconds
        }
    }
    
    pub async fn start_monitoring(&mut self) -> Result<()> {
        println!("Starting background monitoring...");
        
        // Set the running flag
        {
            let mut is_running = self.is_running.lock().unwrap();
            *is_running = true;
        }
        
        let is_running = Arc::clone(&self.is_running);
        
        // Start the monitoring loop
        while *is_running.lock().unwrap() {
            self.check_active_application().await?;
            time::sleep(self.sampling_interval).await;
        }
        
        Ok(())
    }
    
    pub fn stop_monitoring(&mut self) {
        let mut is_running = self.is_running.lock().unwrap();
        *is_running = false;
        println!("Background monitoring stopped.");
    }
    
    async fn check_active_application(&mut self) -> Result<()> {
        // First, detect the foreground app
        let app_name = self.get_foreground_app();
        println!("Debug: Detected foreground app: {}", app_name);
        let now = Instant::now();
        
        // Check if this is a browser and try to get URL info
        let mut browser_url = None;
        let app_lower = app_name.to_lowercase();
        println!("Debug: Checking if app is a browser: {}", app_name);
        if app_lower.contains("chrome") || 
           app_lower.contains("safari") || 
           app_lower.contains("firefox") ||
           app_lower.contains("edge") ||
           app_lower.contains("brave") {
            
            println!("Debug: App detected as browser: {}", app_name);
            if let Some((browser_name, url)) = BrowserMonitor::detect_browser_activity(&app_name) {
                // The real URL is not accessible for privacy reasons, but we can 
                // at least record we detected browser activity
                println!("Debug: Browser activity detected: {} with URL: {}", browser_name, url);
                browser_url = Some(url);
            } else {
                println!("Debug: No browser activity detected for {}", app_name);
            }
        } else {
            println!("Debug: App not detected as browser: {}", app_name);
        }
        
        // If the app changed, record the previous app's duration
        let app_changed = !self.current_app.is_empty() && self.current_app != app_name;
        let url_changed = self.current_browser_url != browser_url;
        let time_threshold_reached = now.duration_since(self.last_save_time) >= Duration::from_secs(30);
        
        println!("Debug: Current app: '{}', New app: '{}'", self.current_app, app_name);
        println!("Debug: Current URL: '{:?}', New URL: '{:?}'", self.current_browser_url, browser_url);
        println!("Debug: app_changed: {}, url_changed: {}, time_threshold_reached: {}", 
                 app_changed, url_changed, time_threshold_reached);
        
        if app_changed || url_changed || time_threshold_reached {
            let duration = now.duration_since(self.last_switch_time);
            
            // Update the app duration in our tracking map
            *self.app_durations.entry(self.current_app.clone()).or_insert(Duration::from_secs(0)) += duration;
            
            // Log the app switch or URL change
            if app_changed {
                println!("Switched from {} to {} (used for {:?})", 
                    self.current_app, app_name, duration);
            } else if url_changed {
                println!("Same browser ({}) but URL changed (used for {:?})",
                    app_name, duration);
            }
            
            // Record completed activity
            let activity_name = if let Some(_url) = &self.current_browser_url {
                // Use browser + URL indicator for naming
                format!("{} - Web Browsing", self.current_app)
            } else {
                self.current_app.clone()
            };
            
            self.save_activity(&activity_name, duration)?;
            self.last_save_time = now;
        }
        
        // Update the current app and time
        self.current_app = app_name;
        self.current_browser_url = browser_url;
        self.last_switch_time = now;
        
        Ok(())
    }
    
    fn save_activity(&self, app_name: &str, duration: Duration) -> Result<()> {
        let category = self.categorize_app(app_name);
        let is_productive = self.is_app_productive(app_name, &category);
        
        let mut activity = Activity::new(
            app_name.to_string(),
            category,
            vec!["automatic".to_string()],
            is_productive,
        );
        
        // Since this is historical data, set the times manually
        let end_time = Local::now();
        let start_time = end_time - chrono::Duration::from_std(duration).unwrap_or_default();
        activity.start_time = start_time;
        activity.end_time = Some(end_time);
        activity.duration = Some(duration);
        
        // Save to database
        let db = self.db.lock().unwrap();
        db.save_activity(&activity)?;
        
        Ok(())
    }
    
    fn categorize_app(&self, app_name: &str) -> String {
        // Special handling for browsers with URL info
        if let Some(url) = &self.current_browser_url {
            if url != "URL unavailable - for privacy reasons" {
                return BrowserMonitor::categorize_url(url);
            }
        }
        
        // Simple categorization based on app name
        let app_lower = app_name.to_lowercase();
        
        if app_lower.contains("chrome") || app_lower.contains("firefox") || 
           app_lower.contains("safari") || app_lower.contains("edge") || 
           app_lower.contains("brave") {
            return "Browser".to_string();
        } else if app_lower.contains("code") || app_lower.contains("intellij") || 
                  app_lower.contains("xcode") || app_lower.contains("vim") || 
                  app_lower.contains("emacs") {
            return "Development".to_string();
        } else if app_lower.contains("word") || app_lower.contains("excel") || 
                  app_lower.contains("powerpoint") || app_lower.contains("notes") {
            return "Productivity".to_string();
        } else if app_lower.contains("slack") || app_lower.contains("teams") || 
                  app_lower.contains("discord") || app_lower.contains("zoom") {
            return "Communication".to_string();
        } else if app_lower.contains("itunes") || app_lower.contains("spotify") || 
                  app_lower.contains("netflix") || app_lower.contains("youtube") {
            return "Entertainment".to_string();
        }
        
        "Other".to_string()
    }
    
    fn is_app_productive(&self, app_name: &str, category: &str) -> bool {
        // Special handling for browsers with URL info
        if let Some(url) = &self.current_browser_url {
            if url != "URL unavailable - for privacy reasons" {
                return BrowserMonitor::is_url_productive(url, category);
            }
        }
        
        // Simple productivity classification
        match category {
            "Development" | "Productivity" => true,
            "Browser" => {
                // Could be refined with URL tracking for browsers
                true
            },
            "Communication" => true,
            "Entertainment" => false,
            _ => {
                // Default assumption - can be refined
                let app_lower = app_name.to_lowercase();
                !(app_lower.contains("game") || app_lower.contains("play"))
            }
        }
    }
    
    #[cfg(target_os = "macos")]
    fn get_foreground_app(&self) -> String {
        println!("Debug: Attempting to get foreground app using osascript first");
        
        // First approach: Try using osascript to get the frontmost application
        match Command::new("osascript")
            .arg("-e")
            .arg("tell application \"System Events\" to get name of first application process whose frontmost is true")
            .output() 
        {
            Ok(output) => {
                if output.status.success() {
                    let app_name = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    println!("Debug: osascript succeeded, got app name: {}", app_name);
                    if !app_name.is_empty() {
                        return app_name;
                    } else {
                        println!("Debug: osascript returned empty app name, falling back to Objective-C method");
                    }
                } else {
                    let error = String::from_utf8_lossy(&output.stderr);
                    println!("Debug: osascript failed with error: {}", error);
                    println!("Debug: Falling back to Objective-C method");
                }
            },
            Err(e) => {
                println!("Debug: Failed to execute osascript: {}", e);
                println!("Debug: Falling back to Objective-C method");
            }
        }
        
        // Second approach (fallback): Use Objective-C API
        println!("Debug: Attempting to get foreground app using macOS Objective-C API");
        unsafe {
            let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
            println!("Debug: Got shared workspace");
            let app: id = msg_send![workspace, frontmostApplication];
            println!("Debug: frontmostApplication result: {}", if app != nil { "Some app" } else { "nil" });
            if app != nil {
                let app_name: id = msg_send![app, localizedName];
                let app_name_str = NSString::UTF8String(app_name);
                let result = std::ffi::CStr::from_ptr(app_name_str)
                    .to_string_lossy()
                    .into_owned();
                println!("Debug: Got app name via Objective-C: {}", result);
                return result;
            }
        }
        
        println!("Debug: Both approaches failed to get foreground app, returning 'Unknown'");
        "Unknown".to_string()
    }
    
    #[cfg(not(target_os = "macos"))]
    fn get_foreground_app(&self) -> String {
        // For non-macOS platforms, use sysinfo to get a best-effort foreground app
        let mut system = System::new();
        system.refresh_processes();
        
        // Get the process with the highest CPU usage as an approximation
        // This is not perfect but provides a basic fallback
        let mut max_cpu = 0.0;
        let mut foreground_app = "Unknown".to_string();
        
        for (_, process) in system.processes() {
            let cpu_usage = process.cpu_usage();
            if cpu_usage > max_cpu {
                max_cpu = cpu_usage;
                foreground_app = process.name().to_string();
            }
        }
        
        foreground_app
    }
}
