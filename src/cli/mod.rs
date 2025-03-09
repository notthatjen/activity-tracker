use clap::{Parser, Subcommand};
use colored::*;
use crate::tracker::Tracker;
use crate::monitor::AppMonitor;
use crate::db::Database;
use rusqlite::Result;

#[derive(Parser)]
#[command(name = "productivity_tracker")]
#[command(about = "A productivity tracking application", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Start tracking a new activity")]
    Start {
        #[arg(help = "Name of the activity")]
        name: String,
        
        #[arg(short, long, help = "Category of the activity")]
        category: String,
        
        #[arg(short, long, help = "Tags for the activity (comma separated)")]
        tags: Option<String>,
        
        #[arg(short, long, help = "Mark as productive", default_value = "true")]
        productive: bool,
    },
    
    #[command(about = "Stop tracking the current activity")]
    Stop,
    
    #[command(about = "Show the current activity")]
    Current,
    
    #[command(about = "Run in background mode to automatically track application usage")]
    Daemon {
        #[arg(short, long, help = "Sampling interval in seconds", default_value = "5")]
        interval: u64,
    },
}

pub async fn run() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Start { name, category, tags, productive } => {
            let mut tracker = Tracker::new()?;
            let tags_vec = tags
                .unwrap_or_default()
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            
            tracker.start_activity(name.clone(), category.clone(), tags_vec, productive)?;
            println!("{} {} in category {}", "Started".green(), name, category);
        },
        Commands::Stop => {
            let mut tracker = Tracker::new()?;
            tracker.stop_activity()?;
            println!("{}", "Activity stopped".green());
        },
        Commands::Current => {
            let tracker = Tracker::new()?;
            if let Some(activity) = tracker.get_current_activity() {
                println!("{}: {} (Category: {})", "Current activity".green(), activity.name, activity.category);
            } else {
                println!("{}", "No activity is currently being tracked".yellow());
            }
        },
        Commands::Daemon { interval } => {
            println!("{}", "Starting background tracking daemon...".green());
            println!("Press Ctrl+C to stop");
            
            // Initialize database
            let db = Database::new()?;
            
            // Create and start the app monitor
            let mut monitor = AppMonitor::new(db);
            
            // Set custom interval if provided
            if interval > 0 {
                monitor.sampling_interval = std::time::Duration::from_secs(interval);
                println!("Sampling interval set to {} seconds", interval);
            }
            
            // Set up Ctrl+C handler
            let ctrl_c = tokio::signal::ctrl_c();
            
            // Start monitoring task
            let monitoring = monitor.start_monitoring();
            
            // Wait for either Ctrl+C or monitoring to complete
            tokio::select! {
                _ = ctrl_c => {
                    println!("Received termination signal");
                    monitor.stop_monitoring();
                },
                result = monitoring => {
                    if let Err(e) = result {
                        eprintln!("Monitoring error: {}", e);
                    }
                }
            }
            
            println!("Background tracking stopped");
        }
    }
    
    Ok(())
}
