use std::{error::Error, io, env};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use reqwest::Client;
use ratatui::{
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FilterCriteria {
    None,
    UserNotOwns,
    RootNotOwns,
    UserAndRootNotOwns,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SortCriteria {
    Difficulty,
    UserOwns,
    RootOwns,
    Name,
}

struct App {
    machines: Vec<Machine>,
    state: ListState,
    htb_api_key: String, // Hackthebox application key
    client: Client, // Reqwest client instance
    info_message: String, // Message for user
    filter_criteria: FilterCriteria, // Criteria for filtering
    sort_criteria: SortCriteria, // Criteria for sorting
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
            filter_criteria: FilterCriteria::None, // Show all machines by default
            sort_criteria: SortCriteria::Difficulty, // Sort by difficulty by default
        })
    }

    fn next(&mut self) {
        let filtered = self.filtered_machines(); // Get filtered list
        let sorted = self.sorted_machines(filtered); // Get sorted list
        let i = match self.state.selected() {
            Some(i) => {
                if i >= sorted.len().saturating_sub(1) { // Saturating sub prevents underflowing
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
        let filtered = self.filtered_machines(); // Get filtered list
        let sorted = self.sorted_machines(filtered); // Get sorted list
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    sorted.len().saturating_sub(1) // Saturating sub prevents underflowing
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    async fn spawn_machine(&mut self) -> Result<(), Box<dyn Error>> {
        let filtered = self.filtered_machines();
        let sorted = self.sorted_machines(filtered);
        if let Some(selected) = self.state.selected() {
            if selected < sorted.len() { // Check index
                let machine = &sorted[selected]; // Use filtered list
                if machine.is_active() {
                    self.info_message = format!("Machine {} is already active.", machine.name);
                    return Ok(());
                }
            
                let url = format!("{}/vm/spawn/?machine_id={}", HTB_API_URL, machine.id.to_string());
                let res = self.client
                    .post(url)
                    .header("Authorization", format!("Bearer {}", self.htb_api_key))
                    .send()
                    .await?;

                if res.status().is_success() {
                    self.info_message = format!("Spawned machine: {}", machine.name);
                    // Refresh the machine list after spawning
                    // Maybe replace that with update to only one machine (?)
                    self.machines = fetch_all_machines(&self.client, &self.htb_api_key).await?;
                    // After refresh re-apply filter
                    let filtered = self.filtered_machines();
                    let sorted = self.sorted_machines(filtered);
                    if selected < sorted.len() {
                        self.state.select(Some(selected));
                    } else if !sorted.is_empty() {
                        // Index out of bounds, select last item
                        self.state.select(Some(sorted.len() - 1));
                    } else {
                        // List empty
                        self.state.select(None);
                    }
                } else {
                    self.info_message = format!("Failed to spawn {}: {}", machine.name, res.status())
                }
            }
        }
        Ok(())
    }

    fn filtered_machines(&self) -> Vec<Machine> {
        let mut filtered = self.machines.clone();
        filtered.retain(|machine| { // Remove all elements that do not met criteria
            match self.filter_criteria {
                FilterCriteria::None => true,
                FilterCriteria::UserNotOwns => !machine.auth_user_in_user_owns,
                FilterCriteria::RootNotOwns => !machine.auth_user_in_root_owns,
                FilterCriteria::UserAndRootNotOwns => !machine.auth_user_in_user_owns && !machine.auth_user_in_root_owns,
            }
        });
        filtered
    }

    fn sorted_machines(&self, machines: Vec<Machine>) -> Vec<Machine> {
        let mut sorted = machines;
        sorted.sort_by(|a, b| {
            match self.sort_criteria {
                SortCriteria::Difficulty => a.difficulty.cmp(&b.difficulty), // Ascending
                SortCriteria::UserOwns => b.user_owns_count.cmp(&a.user_owns_count), // Descending
                SortCriteria::RootOwns => b.root_owns_count.cmp(&a.root_owns_count),
                SortCriteria::Name => a.name.cmp(&b.name)
            }
        });
        sorted
    }

    fn cycle_filter(&mut self) {
        self.filter_criteria = match self.filter_criteria {
            FilterCriteria::None => FilterCriteria::UserNotOwns,
            FilterCriteria::UserNotOwns => FilterCriteria::RootNotOwns,
            FilterCriteria::RootNotOwns => FilterCriteria::UserAndRootNotOwns,
            FilterCriteria::UserAndRootNotOwns => FilterCriteria::None,
        };
        self.state.select(None);
    }

    fn cycle_sort(&mut self) {
        self.sort_criteria = match self.sort_criteria {
            SortCriteria::Difficulty => SortCriteria::UserOwns,
            SortCriteria::UserOwns => SortCriteria::RootOwns,
            SortCriteria::RootOwns => SortCriteria::Name,
            SortCriteria::Name => SortCriteria::Difficulty,
        };
        self.state.select(None);
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
    app: Result<App, Box<dyn std::error::Error>>
) ->io::Result<()> {
    let mut app_unwraped = app.unwrap();
    loop {
        terminal.draw(|f| ui(f, &mut app_unwraped))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('f') => app_unwraped.cycle_filter(),
                    KeyCode::Char('s') => app_unwraped.cycle_sort(),
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

    let filtered_machines = app.filtered_machines();
    let sorted_machines = app.sorted_machines(filtered_machines);

    let items: Vec<ListItem> = sorted_machines
        .iter()
        .map(|machine| {
            let status = if machine.is_active() {
                Span::styled("Active", Style::default().fg(Color::Green))
            } else {
                Span::styled("Inactive", Style::default().fg(Color::Red))
            };
            let user_owns_symbol = if machine.auth_user_in_user_owns {
                "✓"
            } else {
                " "
            };
            let root_owns_symbol = if machine.auth_user_in_root_owns {
                "✓"
            } else {
                " "
            };

            let line = Line::from(vec![
                Span::raw(
                    format!(
                        "{:15} ({:10}) [{:3}] U:{}, R:{}", 
                        machine.name, machine.os, machine.difficulty, user_owns_symbol, root_owns_symbol
                    )
                ),
                status,
            ]);

            ListItem::new(line).style(Style::default().fg(Color::White))
        })
        .collect();

    let list_title = format!(
        "Machines (Filter: {:?}, Sort: {:?})",
        app.filter_criteria, app.sort_criteria
    );
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(list_title))
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
