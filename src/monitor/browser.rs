use std::process::Command;

pub struct BrowserMonitor;

impl BrowserMonitor {
    // Detect browser and potentially return URL
    pub fn detect_browser_activity(app_name: &str) -> Option<(String, String)> {
        println!("Debug: Attempting to detect browser activity for: {}", app_name);
        
        // Check for specific browser based on app_name first
        let lower_app_name = app_name.to_lowercase();
        
        // Prioritize checking for the browser that's in focus based on app_name
        if lower_app_name.contains("brave") {
            println!("Debug: Prioritizing Brave check based on app name");
            if let Some(browser_info) = Self::detect_brave() {
                println!("Debug: Brave detected, returning info");
                return Some(browser_info);
            }
        } else if lower_app_name.contains("chrome") {
            println!("Debug: Prioritizing Chrome check based on app name");
            if let Some(browser_info) = Self::detect_chrome() {
                println!("Debug: Chrome detected, returning info");
                return Some(browser_info);
            }
        } else if lower_app_name.contains("safari") {
            println!("Debug: Prioritizing Safari check based on app name");
            if let Some(browser_info) = Self::detect_safari() {
                println!("Debug: Safari detected, returning info");
                return Some(browser_info);
            }
        }
        
        // If we didn't match the specific browser, try the other browsers in order
        if !lower_app_name.contains("chrome") {
            println!("Debug: Trying Chrome detection");
            if let Some(browser_info) = Self::detect_chrome() {
                println!("Debug: Chrome detected, returning info");
                return Some(browser_info);
            }
        }
        
        if !lower_app_name.contains("safari") {
            println!("Debug: Trying Safari detection");
            if let Some(browser_info) = Self::detect_safari() {
                println!("Debug: Safari detected, returning info");
                return Some(browser_info);
            }
        }
        
        if !lower_app_name.contains("brave") {
            println!("Debug: Trying Brave detection");
            if let Some(browser_info) = Self::detect_brave() {
                println!("Debug: Brave detected, returning info");
                return Some(browser_info);
            }
        }
        
        println!("Debug: No browsers detected");
        None
    }
    
    #[cfg(target_os = "macos")]
    fn detect_chrome() -> Option<(String, String)> {
        println!("Debug: Attempting to detect Chrome browser");
        // For demo purposes, just detect if Chrome is running - actual implementation would
        // require a more complex approach to safely access Chrome's history or cookies
        
        // More specific check for Chrome to avoid detecting Brave
        let output = Command::new("pgrep")
            .arg("-if")
            .arg("Google Chrome$")
            .output();
            
        match output {
            Ok(output) => {
                let stdout_len = output.stdout.len();
                println!("Debug: Chrome pgrep output length: {}", stdout_len);
                if stdout_len > 0 {
                    println!("Debug: Chrome detected as running");
                    return Some(("Google Chrome".to_string(), "URL unavailable - for privacy reasons".to_string()));
                } else {
                    println!("Debug: Chrome not detected as running");
                }
            },
            Err(e) => println!("Debug: Error running pgrep for Chrome: {}", e),
        }
        
        None
    }
    
    #[cfg(not(target_os = "macos"))]
    fn detect_chrome() -> Option<(String, String)> {
        None
    }
    
    #[cfg(target_os = "macos")]
    fn detect_safari() -> Option<(String, String)> {
        println!("Debug: Attempting to detect Safari browser");
        // Check if Safari is running
        let output = Command::new("pgrep")
            .arg("-i")
            .arg("Safari")
            .output();
            
        match output {
            Ok(output) => {
                let stdout_len = output.stdout.len();
                println!("Debug: Safari pgrep output length: {}", stdout_len);
                if stdout_len > 0 {
                    println!("Debug: Safari detected as running");
                    return Some(("Safari".to_string(), "URL unavailable - for privacy reasons".to_string()));
                } else {
                    println!("Debug: Safari not detected as running");
                }
            },
            Err(e) => println!("Debug: Error running pgrep for Safari: {}", e),
        }
        
        None
    }
    
    #[cfg(not(target_os = "macos"))]
    fn detect_safari() -> Option<(String, String)> {
        None
    }
    
    #[cfg(target_os = "macos")]
    fn detect_brave() -> Option<(String, String)> {
        println!("Debug: Attempting to detect Brave browser");
        // Check if Brave is running with more specific pattern
        let output = Command::new("pgrep")
            .arg("-if")
            .arg("Brave Browser$")
            .output();
            
        match output {
            Ok(output) => {
                let stdout_len = output.stdout.len();
                println!("Debug: Brave pgrep output length: {}", stdout_len);
                if stdout_len > 0 {
                    println!("Debug: Brave detected as running");
                    return Some(("Brave Browser".to_string(), "URL unavailable - for privacy reasons".to_string()));
                } else {
                    println!("Debug: Brave not detected as running");
                }
            },
            Err(e) => println!("Debug: Error running pgrep for Brave: {}", e),
        }
        
        None
    }
    
    #[cfg(not(target_os = "macos"))]
    fn detect_brave() -> Option<(String, String)> {
        None
    }
    
    // For future enhancement: categorize URLs based on domain
    pub fn categorize_url(url: &str) -> String {
        let domain = url.to_lowercase();
        
        if domain.contains("github.com") {
            return "Development".to_string();
        } else if domain.contains("stackoverflow.com") || domain.contains("docs.") {
            return "Research".to_string();
        } else if domain.contains("linkedin.com") || domain.contains("twitter.com") || domain.contains("facebook.com") {
            return "Social Media".to_string();
        } else if domain.contains("youtube.com") || domain.contains("netflix.com") {
            return "Entertainment".to_string();
        } else if domain.contains("mail.") || domain.contains("gmail.com") || domain.contains("outlook.com") {
            return "Communication".to_string();
        }
        
        "Web Browsing".to_string()
    }
    
    // Judge if a URL is likely to be productive
    pub fn is_url_productive(url: &str, category: &str) -> bool {
        match category {
            "Development" | "Research" | "Communication" => true,
            "Entertainment" | "Social Media" => false,
            _ => {
                // Default heuristic
                !url.contains("game") && !url.contains("play") && !url.contains("video")
            }
        }
    }
}
