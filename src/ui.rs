// src/ui.rs
use ratatui::{
    Frame,
    layout::{Layout, Direction, Constraint, Rect, Alignment},
    style::{Color, Style, Modifier},
    widgets::{Block, Borders, Paragraph, Table, Row, Cell, BorderType},
};

use crate::app::App;
use crate::state::format_game_time;

pub fn render(frame: &mut Frame, app: &App) {
    // Create the layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Length(3),  // Game time
            Constraint::Min(10),    // Enemy heroes
            Constraint::Length(2),  // Status bar
        ].as_ref())
        .split(frame.area());
    
    // Render title
    render_title(frame, chunks[0]);
    
    // Render game time
    render_game_time(frame, chunks[1], app.game_time);
    
    // Render enemy heroes
    render_enemy_heroes(frame, chunks[2], &app.enemy_heroes);
    
    // Render status bar
    render_status_bar(frame, chunks[3]);
}

fn render_title(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new("Dota 2 Coach - Enemy Tracker")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded))
        .alignment(Alignment::Center);
    
    frame.render_widget(title, area);
}

fn render_game_time(frame: &mut Frame, area: Rect, game_time: i32) {
    let formatted_time = format_game_time(Some(game_time));
    let time_text = format!("Game Time: {}", formatted_time);
    
    let game_time_widget = Paragraph::new(time_text)
        .style(Style::default().fg(Color::Green))
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Time"))
        .alignment(Alignment::Center);
    
    frame.render_widget(game_time_widget, area);
}

fn render_enemy_heroes(frame: &mut Frame, area: Rect, enemy_heroes: &std::collections::HashMap<String, crate::state::EnemyHero>) {
    if enemy_heroes.is_empty() {
        let no_data = Paragraph::new("No enemy heroes detected yet. Waiting for data...")
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Enemy Heroes"))
            .alignment(Alignment::Center);
        
        frame.render_widget(no_data, area);
        return;
    }
    
    // Render enemy heroes in a table
    let header_cells = ["Hero", "Level", "Health", "Mana", "Last Seen", "Position"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).style(Style::default().add_modifier(Modifier::BOLD));
    
    let rows = enemy_heroes.iter().map(|(_, hero)| {
        let last_seen = format_game_time(Some(hero.last_seen));
        let position = format!("({}, {})", hero.position.0, hero.position.1);
        
        // Format health information
        let health_display = match (hero.health, hero.health_percent) {
            (Some(health), Some(percent)) => format!("{}/{} ({}%)", health, hero.max_health.unwrap_or(0), percent),
            (_, Some(percent)) => format!("{}%", percent),
            _ => "Unknown".to_string()
        };
        
        // Format mana information
        let mana_display = match (hero.mana, hero.mana_percent) {
            (Some(mana), Some(percent)) => format!("{}/{} ({}%)", mana, hero.max_mana.unwrap_or(0), percent),
            (_, Some(percent)) => format!("{}%", percent),
            _ => "Unknown".to_string()
        };
        
        // Add color based on health percentage
        let health_style = if let Some(percent) = hero.health_percent {
            if percent < 25 {
                Style::default().fg(Color::Red)
            } else if percent < 50 {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Green)
            }
        } else {
            Style::default().fg(Color::White)
        };
        
        Row::new([
            Cell::from(hero.name.clone()).style(Style::default().fg(Color::Cyan)),
            Cell::from(hero.estimated_level.to_string()),
            Cell::from(health_display).style(health_style),
            Cell::from(mana_display).style(Style::default().fg(Color::LightBlue)),
            Cell::from(last_seen),
            Cell::from(position),
        ])
    });
    
    let widths = [
        Constraint::Percentage(20),  // Hero
        Constraint::Percentage(8),   // Level
        Constraint::Percentage(18),  // Health
        Constraint::Percentage(18),  // Mana
        Constraint::Percentage(16),  // Last Seen
        Constraint::Percentage(20),  // Position
    ];
    
    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().title("Enemy Heroes").borders(Borders::ALL).border_type(BorderType::Rounded))
        .column_spacing(1)
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    
    frame.render_widget(table, area);
}

fn render_status_bar(frame: &mut Frame, area: Rect) {
    let status = Paragraph::new("Press q to quit")
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center);
    
    frame.render_widget(status, area);
}