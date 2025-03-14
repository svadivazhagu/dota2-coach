// src/event.rs
use std::time::Duration;
use color_eyre::eyre::OptionExt;
use futures::{FutureExt, StreamExt};
use crossterm::event::{Event as CrosstermEvent};
use tokio::sync::mpsc;

// The frequency at which tick events are emitted
const TICK_FPS: f64 = 30.0;

// Representation of all possible events
#[derive(Debug)]
pub enum Event {
    /// An event that is emitted on a regular schedule for UI updates
    Tick,
    /// Crossterm terminal events
    Crossterm(CrosstermEvent),
    /// Custom application events
    App(AppEvent),
}

// Application-specific events
#[derive(Debug)]
pub enum AppEvent {
    Quit,
    // Add more app-specific events as needed
}

// Terminal event handler
pub struct EventHandler {
    sender: mpsc::UnboundedSender<Event>,
    receiver: mpsc::UnboundedReceiver<Event>,
}

impl EventHandler {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let event_sender = sender.clone();
        
        // Spawn a task to handle events
        tokio::spawn(async move {
            let tick_rate = Duration::from_secs_f64(1.0 / TICK_FPS);
            let mut reader = crossterm::event::EventStream::new();
            let mut tick = tokio::time::interval(tick_rate);
            
            loop {
                let tick_delay = tick.tick();
                let crossterm_event = reader.next().fuse();
                
                tokio::select! {
                    _ = event_sender.closed() => {
                        break;
                    }
                    _ = tick_delay => {
                        let _ = event_sender.send(Event::Tick);
                    }
                    Some(Ok(evt)) = crossterm_event => {
                        let _ = event_sender.send(Event::Crossterm(evt));
                    }
                }
            }
        });
        
        Self { sender, receiver }
    }
    
    pub async fn next(&mut self) -> color_eyre::Result<Event> {
        self.receiver.recv().await.ok_or_eyre("Failed to receive event")
    }
    
    pub fn send(&self, event: AppEvent) {
        let _ = self.sender.send(Event::App(event));
    }
}