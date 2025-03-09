use std::process::Command;
use std::str;
use url::Url;

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
    
    // Extract domain from URL
    fn extract_domain(url_str: &str) -> String {
        println!("Debug: Extracting domain from URL: {}", url_str);
        
        if url_str.is_empty() || url_str == "URL unavailable" {
            return "unknown.domain".to_string();
        }
        
        match Url::parse(url_str) {
            Ok(url) => {
                match url.host_str() {
                    Some(host) => {
                        println!("Debug: Extracted domain: {}", host);
                        host.to_string()
                    },
                    None => {
                        println!("Debug: No host in URL");
                        "unknown.domain".to_string()
                    }
                }
            },
            Err(e) => {
                println!("Debug: Error parsing URL {}: {}", url_str, e);
                "unknown.domain".to_string()
            }
        }
    }
    
    // Run AppleScript and return the output
    fn run_applescript(script: &str) -> Result<String, String> {
        println!("Debug: Running AppleScript: {}", script);
        
        let output = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output();
            
        match output {
            Ok(output) => {
                if output.status.success() {
                    let stdout = match str::from_utf8(&output.stdout) {
                        Ok(s) => s.trim().to_string(),
                        Err(_) => return Err("Invalid UTF-8 in AppleScript output".to_string()),
                    };
                    println!("Debug: AppleScript output: {}", stdout);
                    Ok(stdout)
                } else {
                    let stderr = match str::from_utf8(&output.stderr) {
                        Ok(s) => s.trim().to_string(),
                        Err(_) => "Unknown error".to_string(),
                    };
                    println!("Debug: AppleScript error: {}", stderr);
                    Err(stderr)
                }
            },
            Err(e) => {
                println!("Debug: Error running AppleScript: {}", e);
                Err(e.to_string())
            }
        }
    }

    #[cfg(target_os = "macos")]
    fn detect_chrome() -> Option<(String, String)> {
        println!("Debug: Attempting to detect Chrome browser");
        
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
                    
                    // Try to get the URL from Chrome using AppleScript
                    let script = r#"
                        tell application "Google Chrome"
                            if it is running then
                                try
                                    get URL of active tab of first window
                                on error
                                    return "URL unavailable"
                                end try
                            else
                                return "Chrome not running"
                            end if
                        end tell
                    "#;
                    
                    let url = match Self::run_applescript(script) {
                        Ok(url) if !url.is_empty() && url != "Chrome not running" => {
                            let domain = Self::extract_domain(&url);
                            domain
                        },
                        _ => "URL unavailable".to_string()
                    };
                    
                    return Some(("Google Chrome".to_string(), url));
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
                    
                    // Try to get the URL from Safari using AppleScript
                    let script = r#"
                        tell application "Safari"
                            if it is running then
                                try
                                    get URL of current tab of first window
                                on error
                                    return "URL unavailable"
                                end try
                            else
                                return "Safari not running"
                            end if
                        end tell
                    "#;
                    
                    let url = match Self::run_applescript(script) {
                        Ok(url) if !url.is_empty() && url != "Safari not running" => {
                            let domain = Self::extract_domain(&url);
                            domain
                        },
                        _ => "URL unavailable".to_string()
                    };
                    
                    return Some(("Safari".to_string(), url));
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
                    
                    // Try to get the URL from Brave Browser using AppleScript
                    // Brave is Chromium-based so we can use similar script as Chrome
                    let script = r#"
                        tell application "Brave Browser"
                            if it is running then
                                try
                                    get URL of active tab of first window
                                on error
                                    return "URL unavailable"
                                end try
                            else
                                return "Brave not running"
                            end if
                        end tell
                    "#;
                    
                    let url = match Self::run_applescript(script) {
                        Ok(url) if !url.is_empty() && url != "Brave not running" => {
                            let domain = Self::extract_domain(&url);
                            domain
                        },
                        _ => "URL unavailable".to_string()
                    };
                    
                    return Some(("Brave Browser".to_string(), url));
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
    
    // Categorize URLs based on domain into specific categories
    pub fn categorize_url(url: &str) -> String {
        let domain = url.to_lowercase();
        
        println!("Debug: Categorizing domain: {}", domain);
        
        // Development related sites
        if domain.contains("github.com") || 
           domain.contains("gitlab.com") ||
           domain.contains("bitbucket.org") || 
           domain.contains("stackoverflow.com") || 
           domain.contains("dev.to") ||
           domain.contains("medium.com/programming") ||
           domain.contains("hackernews") ||
           domain.contains("codeacademy.com") ||
           domain.contains("replit.com") ||
           domain.contains("codepen.io") ||
           domain.contains("localhost") ||
           domain.contains("127.0.0.1") ||
           domain == "localhost" {
            return "Development".to_string();
        } 
        
        // Productivity tools and sites
        else if domain.contains("linear.app") ||
                domain.contains("notion.so") ||
                domain.contains("trello.com") ||
                domain.contains("asana.com") ||
                domain.contains("monday.com") ||
                domain.contains("clickup.com") ||
                domain.contains("todoist.com") ||
                domain.contains("evernote.com") ||
                domain.contains("airtable.com") ||
                domain.contains("miro.com") ||
                domain.contains("figma.com") ||
                domain.contains("docs.google.com") ||
                domain.contains("sheets.google.com") ||
                domain.contains("meet.google.com") ||
                domain.contains("drive.google.com") ||
                domain.contains("calendar.google.com") ||
                domain.contains("basecamp.com") {
            return "Productivity".to_string();
        }
        
        // Documentation and learning
        else if domain.contains("docs.") ||
                domain.contains("documentation") ||
                domain.contains("learn.") ||
                domain.contains("courses") ||
                domain.contains("udemy.com") ||
                domain.contains("coursera.org") ||
                domain.contains("edx.org") ||
                domain.contains("pluralsight.com") ||
                domain.contains("freecodecamp.org") ||
                domain.contains("khanacademy.org") {
            return "Research".to_string();
        }
        
        // Social Media
        else if domain.contains("twitter.com") ||
                domain.contains("x.com") ||
                domain.contains("facebook.com") ||
                domain.contains("instagram.com") ||
                domain.contains("reddit.com") ||
                domain.contains("linkedin.com") ||
                domain.contains("tiktok.com") ||
                domain.contains("snapchat.com") ||
                domain.contains("pinterest.com") ||
                domain.contains("discord.com") ||
                domain.contains("whatsapp.com") ||
                domain.contains("telegram.org") ||
                domain.contains("slack.com") {
            return "Social Media".to_string();
        }
        
        // Entertainment
        else if domain.contains("youtube.com") ||
                domain.contains("netflix.com") ||
                domain.contains("hulu.com") ||
                domain.contains("disneyplus.com") ||
                domain.contains("hbomax.com") ||
                domain.contains("primevideo.com") ||
                domain.contains("twitch.tv") ||
                domain.contains("vimeo.com") ||
                domain.contains("spotify.com") ||
                domain.contains("soundcloud.com") ||
                domain.contains("apple.com/music") ||
                domain.contains("deezer.com") ||
                domain.contains("pandora.com") ||
                domain.contains("tidal.com") {
            return "Entertainment".to_string();
        }
        
        // News and Information
        else if domain.contains("news.") ||
                domain.contains("bbc.") ||
                domain.contains("cnn.com") ||
                domain.contains("nytimes.com") ||
                domain.contains("wsj.com") ||
                domain.contains("reuters.com") ||
                domain.contains("bloomberg.com") ||
                domain.contains("economist.com") ||
                domain.contains("ft.com") ||
                domain.contains("forbes.com") ||
                domain.contains("washingtonpost.com") ||
                domain.contains("apnews.com") ||
                domain.contains("theguardian.com") ||
                domain.contains("huffpost.com") {
            return "Information".to_string();
        }
        
        // Email and Communication
        else if domain.contains("mail.") || 
                domain.contains("gmail.com") || 
                domain.contains("outlook.com") ||
                domain.contains("yahoo.mail") ||
                domain.contains("protonmail.com") ||
                domain.contains("zoho.com/mail") ||
                domain.contains("icloud.com") ||
                domain.contains("fastmail.com") ||
                domain.contains("tutanota.com") ||
                domain.contains("mail.ru") {
            return "Communication".to_string();
        }
        
        // Default category for uncategorized sites
        "Web Browsing".to_string()
    }
    
    // Judge if a URL is likely to be productive
    // Determine if a URL is likely to be productive based on its category and content
    pub fn is_url_productive(url: &str, category: &str) -> bool {
        match category {
            // Productive categories
            "Development" | "Research" | "Communication" | "Productivity" | "Information" => true,
            
            // Non-productive categories
            "Entertainment" | "Social Media" => false,
            
            // For uncategorized URLs, use heuristics
            _ => {
                // Check for obviously non-productive keywords
                let non_productive_keywords = [
                    "game", "play", "video", "stream", "watch", "movie", "entertainment",
                    "meme", "fun", "funny", "joke", "shopping", "sale", "discount"
                ];
                
                // Check for productive keywords
                let productive_keywords = [
                    "work", "job", "task", "project", "learn", "study", "code", "document",
                    "research", "paper", "article", "analysis", "book", "course", "tutorial"
                ];
                
                // Check if any non-productive keywords are in the URL
                let has_non_productive = non_productive_keywords.iter()
                    .any(|&keyword| url.contains(keyword));
                
                // Check if any productive keywords are in the URL
                let has_productive = productive_keywords.iter()
                    .any(|&keyword| url.contains(keyword));
                
                // If has productive keywords and no non-productive ones, likely productive
                // Otherwise, default to not productive
                has_productive && !has_non_productive
            }
        }
}
