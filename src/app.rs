// src/app.rs
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use crate::event::{Event, EventHandler, AppEvent};
use crate::state::{GameState, EnemyHero, extract_enemy_heroes};
use crate::ui;

pub struct App {
    pub running: bool,
    pub game_state: Arc<Mutex<Option<GameState>>>,
    pub last_game_state: Arc<Mutex<Option<GameState>>>,
    pub enemy_heroes: HashMap<String, EnemyHero>,
    pub events: EventHandler,
    pub game_time: i32,
}

impl App {
    pub fn new() -> Self {
        Self {
            running: true,
            game_state: Arc::new(Mutex::new(None)),
            last_game_state: Arc::new(Mutex::new(None)),
            enemy_heroes: HashMap::new(),
            events: EventHandler::new(),
            game_time: 0,
        }
    }
    
    pub async fn run(mut self, mut terminal: Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
        while self.running {
            // Update application state
            self.update();
            
            // Render UI
            terminal.draw(|frame| ui::render(frame, &self))?;
            
            // Handle events
            match self.events.next().await? {
                Event::Tick => {}
                Event::Crossterm(event) => {
                    if let crossterm::event::Event::Key(key_event) = event {
                        self.handle_key_event(key_event)?;
                    }
                }
                Event::App(app_event) => {
                    self.handle_app_event(app_event);
                }
            }
        }
        
        Ok(())
    }
    
    fn update(&mut self) {
        // Get the current game state
        let game_state_option = {
            let gs = self.game_state.lock().unwrap();
            gs.clone()
        };
        
        if let Some(game_state) = game_state_option {
            // Update game time
            self.game_time = game_state.map.as_ref()
                .and_then(|m| m.game_time)
                .unwrap_or(0);
            
            // Debug output every 30 seconds to find health data
            if self.game_time % 30 == 0 {
                crate::state::debug_game_state(&game_state);
                crate::state::explore_gsi_data(&game_state);
            }
            
            // Update enemy heroes
            let new_enemy_heroes = extract_enemy_heroes(&game_state);
            
            // Merge new information with existing data
            for (name, hero) in new_enemy_heroes {
                // Update or insert the enemy hero info
                self.enemy_heroes
                    .entry(name.clone())
                    .and_modify(|e| {
                        // Only update if we have more recent information
                        if hero.last_seen > e.last_seen {
                            e.position = hero.position;
                            e.last_seen = hero.last_seen;
                            e.estimated_level = hero.estimated_level;
                            
                            // Update health/mana information if available
                            if hero.health.is_some() {
                                e.health = hero.health;
                                e.max_health = hero.max_health;
                                e.health_percent = hero.health_percent;
                            }
                            
                            if hero.mana.is_some() {
                                e.mana = hero.mana;
                                e.max_mana = hero.max_mana;
                                e.mana_percent = hero.mana_percent;
                            }
                        }
                    })
                    .or_insert(hero);
            }
        }
    }
    
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.events.send(AppEvent::Quit);
            }
            KeyCode::Char('c') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.events.send(AppEvent::Quit);
            }
            _ => {}
        }
        
        Ok(())
    }
    
    fn handle_app_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::Quit => {
                self.running = false;
            }
        }
    }
}