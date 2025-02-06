use std::{error::Error, io, env};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use reqwest::Client;
use ratatui::{
    prelude::*,
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Line},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

const HTB_API_URL: &str = "https://labs.hackthebox.com/api/v4";

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Root {
    data: Vec<Machine>,
    links: Link
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Link {
    first: String,
    last: String,
    prev: Option<String>,
    next: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Machine {
    // Add more fields as needed
    id: u64,
    name: String,
    os: String,
    points: u64,
    star: f64,
    release: String,
    difficulty: u64,
    #[serde(rename="user_owns_count")]
    user_owns_count: u64,
    auth_user_in_user_owns: bool,
    #[serde(rename="root_owns_count")]
    root_owns_count: u64,
    auth_user_in_root_owns: bool,
    active: Value,
}

impl Machine {
    fn is_active(&self) -> bool {
        match &self.active {
            Value::Bool(b) => *b,
            Value::Number(n) => n.as_i64() == Some(1),
            _ => false
        }
    }
}

struct App {
    machines: Vec<Machine>,
    state: ListState,
    htb_api_key: String, // Hackthebox application key
    client: Client, // Reqwest client instance
    info_message: String, // Message for user
}

impl App {
    // Create new application and accept Hackthebox application key
    async fn new(htb_api_key: String) -> Result<App, Box<dyn Error>> {
        let client = reqwest::Client::new();
        let machines = fetch_all_machines(&client, &htb_api_key).await?;
        Ok(App {
            machines,
            state: ListState::default(),
            htb_api_key,
            client,
            info_message: String::new(),
        })
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.machines.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.machines.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    async fn spawn_machine(&mut self) -> Result<(), Box<dyn Error>> {
        todo!();
    }
}

async fn fetch_all_machines(client: &Client, htb_api_key: &str) -> Result<Vec<Machine>, Box<dyn Error>> {
    let mut all_machines = Vec::new();

    // Fetch active machines
    let url = format!("{}/machine/paginated?per_page=100", HTB_API_URL);
    let res = fetch_machines(client, htb_api_key, &url).await?;
    all_machines.extend(res.data);

    // Fetch retired machines
    let url = format!("{}/machine/list/retired/paginated?per_page=100", HTB_API_URL);
    let mut res = fetch_machines(client, htb_api_key, &url).await?;
    all_machines.extend(res.data);

    while let Some(next_url) = res.links.next {
        res = fetch_machines(client, htb_api_key, &next_url).await?;
        all_machines.extend(res.data);
    }

    Ok(all_machines)
}

async fn fetch_machines(client: &Client, htb_api_key: &str, url: &str) -> Result<Root, Box<dyn Error>> {
    let res = client
        .get(url)
        .header("Authorization", format!("Bearer {}", htb_api_key))
        .send()
        .await?
        .json::<Root>()
        .await?;
    Ok(res)
}

#[tokio::main]
async fn main() ->Result<(), Box<dyn std::error::Error>> {
    // Setup cross terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create application
    let htb_api_key = env::var("HTB_API_KEY")?;
    let app = App::new(htb_api_key).await;
    let res = run_app(&mut terminal, app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Error handling
    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>, 
    mut app: Result<App, Box<dyn std::error::Error>>
) ->io::Result<()> {
    let mut app_unwraped = app.unwrap();
    loop {
        terminal.draw(|f| ui(f, &mut app_unwraped))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Down => app_unwraped.next(),
                    KeyCode::Up => app_unwraped.previous(),
                    KeyCode::Enter => {
                        let spawn_result = app_unwraped.spawn_machine().await;
                        if let Err(e) = spawn_result {
                            app_unwraped.info_message = format!("Error: {}", e);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks =
        Layout::vertical([Constraint::Percentage(90), Constraint::Percentage(10)]).split(f.area());

    let items: Vec<ListItem> = app
        .machines
        .iter()
        .map(|machine| {
            let status = if machine.is_active() {
                Span::styled("Active", Style::default().fg(Color::Green))
            } else {
                Span::styled("Inactive", Style::default().fg(Color::Red))
            };
            let line = Line::from(vec![
                Span::raw(format!("{:15} ({:10}) ", machine.name, machine.os)),
                status,
            ]);

            ListItem::new(line).style(Style::default().fg(Color::White))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Machines"))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    f.render_stateful_widget(list, chunks[0], &mut app.state);

    let info_paragraph = Paragraph::new(app.info_message.clone())
        .style(Style::default().fg(Color::LightCyan))
        .block(Block::default().borders(Borders::ALL).title("Info"));

    f.render_widget(info_paragraph, chunks[1]);
}
