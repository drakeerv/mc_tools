use crossterm::ExecutableCommand;
use inputbot::{KeybdKey, MouseButton};
use rand::Rng;
use ratatui::prelude::Stylize;
use serde::{Deserialize, Serialize};
use std::io::stdout;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

// select a key from 1 to 9 and press it
// const MAX_KEY: u8 = 9;
// const MIN_KEY: u8 = 1;

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Config {
    min_key: u8,
    max_key: u8,
}

// on value change, save the config to the file
impl Config {
    fn save(&self, file: &str) {
        std::fs::write(file, serde_json::to_string_pretty(&self).unwrap()).unwrap();
    }

    fn load(file: &str) -> Self {
        match std::fs::read_to_string(file) {
            Ok(config) => {
                let config: Config = serde_json::from_str(&config).unwrap();
                config
            }
            Err(_) => {
                let config = Config {
                    min_key: 1,
                    max_key: 9,
                };
                config.save(file);
                config
            }
        }
    }
}

// Current app state
#[derive(Debug, Clone)]
struct State {
    enabled: bool,
    config: Config,
}

#[tokio::main]
async fn main() {
    // Create a shared state that can be accessed from multiple threads.
    let state = Arc::new(Mutex::new(State {
        enabled: false,
        config: Config::load("config.json"),
    }));

    // Get project verion
    let version = env!("CARGO_PKG_VERSION");

    // When the F12 key is pressed, toggle the enabled state.
    let state_ref: Arc<Mutex<State>> = Arc::clone(&state);
    KeybdKey::F12Key.bind(move || {
        let mut state = state_ref.lock().unwrap();
        state.enabled = !state.enabled;
    });

    // When the right mouse button is pressed, generate a random number between 1 and 9 and press the corresponding key.
    let state_ref: Arc<Mutex<State>> = Arc::clone(&state);
    MouseButton::RightButton.bind(move || {
        let state = state_ref.lock().unwrap();
        let mut rng = rand::thread_rng();
        while state.enabled && MouseButton::RightButton.is_pressed() {
            // pick a random number between 1 and 9 and press the corresponding key
            let random_number = rng.gen_range(state.config.min_key..=state.config.max_key) as u32;
            inputbot::get_keybd_key(char::from_digit(random_number, 10).unwrap())
                .unwrap()
                .press();

            // sleep
            std::thread::sleep(Duration::from_millis(250));
        }
    });

    // Inputbot thread
    tokio::spawn(async move {
        inputbot::handle_input_events();
    });

    // Set up the terminal
    stdout()
        .execute(crossterm::terminal::EnterAlternateScreen)
        .unwrap();
    crossterm::terminal::enable_raw_mode().unwrap();
    let mut terminal =
        ratatui::prelude::Terminal::new(ratatui::prelude::CrosstermBackend::new(stdout())).unwrap();
    terminal.clear().unwrap();

    // Main loop state
    let mut loop_state = state.lock().unwrap().clone();
    let state_ref: Arc<Mutex<State>> = Arc::clone(&state);
    let last_update = std::time::Instant::now();
    let list_length = 4;
    let mut list_index = 0;
    let mut list_state = ratatui::widgets::ListState::default().with_selected(Some(list_index));

    // Main loop
    loop {
        if last_update.elapsed().as_millis() > 1000 {
            loop_state = state.lock().unwrap().clone();
        }

        terminal
            .draw(|frame| {
                let area = frame.size();

                let vertical = ratatui::layout::Layout::vertical([
                    ratatui::layout::Constraint::Min(6),
                    ratatui::layout::Constraint::Length(20),
                ]);
                let [header_area, content_area] = vertical.areas(area);

                let vertical = ratatui::layout::Layout::vertical([
                    ratatui::layout::Constraint::Min(4),
                    ratatui::layout::Constraint::Length(20),
                ]);
                let [title_area, credits_area] = vertical.areas(header_area);

                // title
                // frame.render_widget(ratatui::widgets::Paragraph::new("MC Tools").bold().white().centered(), title_area);
                frame.render_widget(
                    tui_big_text::BigText::builder()
                        .pixel_size(tui_big_text::PixelSize::HalfHeight)
                        .lines(vec![
                            "MC Tools".white().into()
                        ])
                        .build()
                        .unwrap(),
                        title_area,
                );

                frame.render_widget(
                    ratatui::widgets::Paragraph::new(format!("Version {} - Made by drakeerv - (Q to quit)", version)).white(),
                    credits_area,
                );

                let list = ratatui::widgets::List::new(vec![
                    ratatui::widgets::ListItem::new(format!("Enabled: {}", loop_state.enabled)),
                    ratatui::widgets::ListItem::new(format!("Min Key: {}", loop_state.config.min_key)),
                    ratatui::widgets::ListItem::new(format!("Max Key: {}", loop_state.config.max_key)),
                    ratatui::widgets::ListItem::new(format!("Save to File")),
                ])
                .white()
                .highlight_symbol(">> ")
                .highlight_style(ratatui::style::Style::default().yellow());

                if loop_state.enabled {
                    frame.render_stateful_widget(list.on_green(), content_area, &mut list_state);
                } else {
                    frame.render_stateful_widget(list.on_red(), content_area, &mut list_state);
                }
            })
            .unwrap();

        if crossterm::event::poll(std::time::Duration::from_millis(16)).unwrap() {
            if let crossterm::event::Event::Key(key) = crossterm::event::read().unwrap() {
                if key.kind == crossterm::event::KeyEventKind::Press {
                    match key.code {
                        crossterm::event::KeyCode::Char('q') => {
                            break;
                        },
                        crossterm::event::KeyCode::Tab => {
                            list_index = (list_index + 1) % list_length;
                            list_state.select(Some(list_index));
                        },
                        crossterm::event::KeyCode::Up => {
                            list_index = (list_index + list_length - 1) % list_length;
                            list_state.select(Some(list_index));
                        },
                        crossterm::event::KeyCode::Down => {
                            list_index = (list_index + 1) % list_length;
                            list_state.select(Some(list_index));
                        },
                        crossterm::event::KeyCode::Left => {
                            let mut state = state_ref.lock().unwrap();
                            match list_index {
                                0 => {
                                    state.enabled = !state.enabled;
                                },
                                1 => {
                                    state.config.min_key = (state.config.min_key + 9) % 10;
                                },
                                2 => {
                                    state.config.max_key = (state.config.max_key + 9) % 10;
                                },
                                _ => {}
                            }
                        },
                        crossterm::event::KeyCode::Right => {
                            let mut state = state_ref.lock().unwrap();
                            match list_index {
                                0 => {
                                    state.enabled = !state.enabled;
                                },
                                1 => {
                                    state.config.min_key = (state.config.min_key + 1) % 10;  
                                },
                                2 => {
                                    state.config.max_key = (state.config.max_key + 1) % 10;
                                },
                                _ => {}
                            }
                        },
                        crossterm::event::KeyCode::Enter => {
                            let mut state = state_ref.lock().unwrap();
                            match list_index {
                                0 => {
                                    state.enabled = !state.enabled;
                                },
                                3 => {
                                    state.config.save("config.json");
                                },
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Finish up
    stdout()
        .execute(crossterm::terminal::LeaveAlternateScreen)
        .unwrap();
    crossterm::terminal::disable_raw_mode().unwrap();
    std::process::exit(0);
}
