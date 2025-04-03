use chrono::{DateTime, Local};
use clap::{Parser, Subcommand};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use notify_rust::Notification;
use rusqlite::{params, Connection, Result as SqlResult};
use std::{
    io,
    time::{Duration, Instant},
};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Gauge, Paragraph},
    Terminal,
};

mod db;
mod timer;
mod ui;
mod analytics;

use timer::{TimerState, TimerType, PomodoroTimer};
use db::Database;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a new Pomodoro session
    Start {
        /// Work duration in minutes
        #[arg(short, long, default_value_t = 25)]
        work: u64,
        
        /// Break duration in minutes
        #[arg(short, long, default_value_t = 5)]
        break_time: u64,
    },
    
    /// Show productivity statistics
    Stats,
    
    /// Export session data to JSON
    Export {
        /// Output file path
        #[arg(short, long)]
        output: Option<String>,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    let db = Database::new()?;
    db.initialize()?;
    
    match &cli.command {
        Some(Commands::Start { work, break_time }) => {
            run_pomodoro_timer(*work, *break_time, &db)?;
        }
        Some(Commands::Stats) => {
            show_stats(&db)?;
        }
        Some(Commands::Export { output }) => {
            export_data(&db, output)?;
        }
        None => {
            // Default to starting with standard 25/5 settings
            run_pomodoro_timer(25, 5, &db)?;
        }
    }
    
    Ok(())
}

fn run_pomodoro_timer(work_mins: u64, break_mins: u64, db: &Database) -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // Create timer
    let work_duration = Duration::from_secs(work_mins * 60);
    let break_duration = Duration::from_secs(break_mins * 60);
    let mut timer = PomodoroTimer::new(work_duration, break_duration);
    
    // Start timer
    timer.start();
    let start_time = Local::now();
    let mut completed_pomodoros = 0;
    
    let mut last_update = Instant::now();
    
    // Main loop
    loop {
        terminal.draw(|f| {
            let size = f.size();
            
            // Create the layout
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Min(0),
                    ]
                    .as_ref(),
                )
                .split(size);
            
            // Title block
            let title = match timer.timer_type() {
                TimerType::Work => format!("🍅 Work Session ({}m)", work_mins),
                TimerType::Break => format!("☕ Break ({}m)", break_mins),
            };
            
            let title_block = Block::default()
                .title(title)
                .borders(Borders::ALL);
            
            f.render_widget(title_block, chunks[0]);
            
            // Timer gauge
            let elapsed = timer.elapsed().as_secs_f64();
            let total = timer.total_time().as_secs_f64();
            let percent = (elapsed / total * 100.0).min(100.0);
            
            let mins_left = ((total - elapsed) / 60.0).ceil() as u64;
            let secs_left = ((total - elapsed) % 60.0).ceil() as u64;
            
            let gauge_label = format!("{:02}:{:02}", mins_left, secs_left);
            
            let gauge = Gauge::default()
                .block(Block::default().borders(Borders::ALL))
                .gauge_style(match timer.timer_type() {
                    TimerType::Work => Style::default().fg(Color::Red),
                    TimerType::Break => Style::default().fg(Color::Green),
                })
                .percent(percent as u16)
                .label(gauge_label);
            
            f.render_widget(gauge, chunks[1]);
            
            // Status
            let status = match timer.state() {
                TimerState::Running => "⏱️  Running",
                TimerState::Paused => "⏸️  Paused",
                TimerState::Stopped => "⏹️  Stopped",
            };
            
            let status_para = Paragraph::new(status)
                .block(Block::default().title("Status").borders(Borders::ALL));
            
            f.render_widget(status_para, chunks[2]);
            
            // Stats
            let stats = format!("🍅 Completed: {}", completed_pomodoros);
            let stats_para = Paragraph::new(stats)
                .block(Block::default().title("Statistics").borders(Borders::ALL));
            
            f.render_widget(stats_para, chunks[3]);
            
            // Help
            let help = vec![
                Spans::from(vec![
                    Span::raw("Press "),
                    Span::styled("p", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to pause/resume, "),
                    Span::styled("r", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to reset, "),
                    Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to quit"),
                ]),
            ];
            
            let help_para = Paragraph::new(help)
                .block(Block::default().title("Help").borders(Borders::ALL));
            
            f.render_widget(help_para, chunks[4]);
        })?;
        
        // Handle elapsed timer
        if timer.state() == TimerState::Running && timer.is_complete() {
            if timer.timer_type() == TimerType::Work {
                // Work session completed
                completed_pomodoros += 1;
                
                // Record completed session in database
                db.save_session(start_time, Local::now(), completed_pomodoros, true)?;
                
                // Show notification
                Notification::new()
                    .summary("Work Session Complete!")
                    .body("Time for a break!")
                    .show()?;
                
                timer.switch_to_break();
            } else {
                // Break session completed
                Notification::new()
                    .summary("Break Complete!")
                    .body("Time to get back to work!")
                    .show()?;
                
                timer.switch_to_work();
            }
        }
        
        // Check for events with small timeout for responsiveness
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        // Save any partial session before quitting
                        if timer.timer_type() == TimerType::Work && timer.elapsed().as_secs() > 60 {
                            db.save_session(
                                start_time,
                                Local::now(),
                                completed_pomodoros,
                                false,
                            )?;
                        }
                        break;
                    }
                    KeyCode::Char('p') => {
                        if timer.state() == TimerState::Running {
                            timer.pause();
                        } else if timer.state() == TimerState::Paused {
                            timer.resume();
                        }
                    }
                    KeyCode::Char('r') => {
                        timer.reset();
                    }
                    _ => {}
                }
            }
        }
        
        // Update every second
        if last_update.elapsed() >= Duration::from_secs(1) {
            timer.update();
            last_update = Instant::now();
        }
    }
    
    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    
    Ok(())
}

fn show_stats(db: &Database) -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Productivity Statistics");
    println!("==========================");
    
    // Get total number of sessions
    let total_sessions: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM sessions", 
        [], 
        |row| row.get(0)
    )?;
    
    // Get completed sessions
    let completed_sessions: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM sessions WHERE completed = 1", 
        [], 
        |row| row.get(0)
    )?;
    
    // Get total focus time
    let total_minutes: i64 = db.conn.query_row(
        "SELECT SUM(CAST((julianday(end_time) - julianday(start_time)) * 24 * 60 As Integer)) 
         FROM sessions", 
        [], 
        |row| row.get(0)
    ).unwrap_or(0);
    
    println!("Total Sessions: {}", total_sessions);
    println!("Completed Sessions: {}", completed_sessions);
    println!("Completion Rate: {}%", 
             if total_sessions > 0 {
                 (completed_sessions as f64 / total_sessions as f64 * 100.0).round()
             } else {
                 0.0
             });
    println!("Total Focus Time: {} hours {} minutes", total_minutes / 60, total_minutes % 60);
    
    // Show most productive days
    println!("\nMost Productive Days:");
    println!("--------------------");
    
    let mut stmt = db.conn.prepare(
        "SELECT date(start_time) as day, 
                COUNT(*) as sessions,
                SUM(CAST((julianday(end_time) - julianday(start_time)) * 24 * 60 As Integer)) as minutes
         FROM sessions 
         GROUP BY day 
         ORDER BY minutes DESC
         LIMIT 5"
    )?;
    
    let days = stmt.query_map([], |row| {
        let day: String = row.get(0)?;
        let sessions: i64 = row.get(1)?;
        let minutes: i64 = row.get(2)?;
        
        Ok((day, sessions, minutes))
    })?;
    
    for day_result in days {
        let (day, sessions, minutes) = day_result?;
        println!("{}: {} sessions, {} hours {} minutes", 
                 day, sessions, minutes / 60, minutes % 60);
    }
    
    println!("\nTip: Run 'rusty_pomodoro export' to get detailed session data");
    
    Ok(())
}

fn export_data(db: &Database, output_path: &Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let mut stmt = db.conn.prepare(
        "SELECT id, start_time, end_time, pomodoro_count, completed
         FROM sessions 
         ORDER BY start_time DESC"
    )?;
    
    #[derive(serde::Serialize)]
    struct Session {
        id: i64,
        start_time: String,
        end_time: String,
        pomodoro_count: i64,
        completed: bool,
        duration_minutes: i64,
    }
    
    let sessions = stmt.query_map([], |row| {
        let id: i64 = row.get(0)?;
        let start_time: String = row.get(1)?;
        let end_time: String = row.get(2)?;
        let pomodoro_count: i64 = row.get(3)?;
        let completed: bool = row.get(4)?;
        
        // Calculate duration in minutes
        let start = DateTime::parse_from_rfc3339(&start_time)
            .unwrap()
            .with_timezone(&Local);
        let end = DateTime::parse_from_rfc3339(&end_time)
            .unwrap()
            .with_timezone(&Local);
        
        let duration = end.signed_duration_since(start);
        let duration_minutes = duration.num_minutes();
        
        Ok(Session {
            id,
            start_time,
            end_time,
            pomodoro_count,
            completed,
            duration_minutes,
        })
    })?;
    
    let mut all_sessions = Vec::new();
    for session in sessions {
        all_sessions.push(session?);
    }
    
    let json = serde_json::to_string_pretty(&all_sessions)?;
    
    match output_path {
        Some(path) => {
            std::fs::write(path, json)?;
            println!("Data exported to {}", path);
        }
        None => {
            println!("{}", json);
        }
    }
    
    Ok(())
}