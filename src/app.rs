use std::error;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use reqwest::Client;
use ratatui::widgets::ListState;

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

const HTB_API_URL: &str = "https://labs.hackthebox.com/api/v4";

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub data: Vec<Machine>,
    pub links: Link
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    pub first: String,
    pub last: String,
    pub prev: Option<String>,
    pub next: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Machine {
    // Add more fields as needed
    pub id: u64,
    pub name: String,
    pub os: String,
    pub points: u64,
    pub star: f64,
    pub release: String,
    pub difficulty: u64,
    #[serde(rename="user_owns_count")]
    pub user_owns_count: u64,
    pub auth_user_in_user_owns: bool,
    #[serde(rename="root_owns_count")]
    pub root_owns_count: u64,
    pub auth_user_in_root_owns: bool,
    pub active: Value,
    pub ip: Option<String>,
}

impl Machine {
    pub fn is_active(&self) -> bool {
        match &self.active {
            Value::Bool(b) => *b,
            Value::Number(n) => n.as_i64() == Some(1),
            _ => false
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterCriteria {
    None,
    UserNotOwns,
    RootNotOwns,
    UserAndRootNotOwns,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortCriteria {
    Difficulty,
    UserOwns,
    RootOwns,
    Name,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Flag,
}

pub struct App {
    pub running: bool,
    pub htb_api_key: String, // Hackthebox application key
    pub client: Client, // Reqwest client

    pub machines: Vec<Machine>,
    pub state: ListState,
    pub info_message: String, // Message for user
    pub filter_criteria: FilterCriteria, // Criteria for filtering
    pub sort_criteria: SortCriteria, // Criteria for sorting
    
    pub input_mode: InputMode, // input mode
    pub flag_input: String,
    pub show_input_field: bool, // control input visibility
    pub selected_machine_ip: Option<String>, // IP of active machine
    pub selected_machine_id: Option<u64>,
}

impl App {
    // Create new application and accept Hackthebox application key
    pub fn new(htb_api_key: String) ->Self {
        let client = reqwest::Client::new();
        let machines = fetch_all_machines(&client, &htb_api_key).await?;
        Self {
            running: true,
            htb_api_key,
            client,
            machines,
            state: ListState::default(),
            info_message: String::new(),
            filter_criteria: FilterCriteria::None, // Show all machines by default
            sort_criteria: SortCriteria::Difficulty, // Sort by difficulty by default
            input_mode: InputMode::Normal,
            flag_input: String::new(),
            show_input_field: false,
            selected_machine_ip: None,
            selected_machine_id: None,
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn next(&mut self) {
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
        self.update_input_fields();
    }

    pub fn previous(&mut self) {
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
        self.update_input_fields();
    }

    pub async fn spawn_machine(&mut self) -> AppResult<()> {
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
            self.update_input_fields();
        }
        Ok(())
    }

    pub fn filtered_machines(&self) -> Vec<Machine> {
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

    pub fn sorted_machines(&self, machines: Vec<Machine>) -> Vec<Machine> {
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

    pub fn cycle_filter(&mut self) {
        self.filter_criteria = match self.filter_criteria {
            FilterCriteria::None => FilterCriteria::UserNotOwns,
            FilterCriteria::UserNotOwns => FilterCriteria::RootNotOwns,
            FilterCriteria::RootNotOwns => FilterCriteria::UserAndRootNotOwns,
            FilterCriteria::UserAndRootNotOwns => FilterCriteria::None,
        };
        self.state.select(None);
        self.update_input_fields();
    }

    pub fn cycle_sort(&mut self) {
        self.sort_criteria = match self.sort_criteria {
            SortCriteria::Difficulty => SortCriteria::UserOwns,
            SortCriteria::UserOwns => SortCriteria::RootOwns,
            SortCriteria::RootOwns => SortCriteria::Name,
            SortCriteria::Name => SortCriteria::Difficulty,
        };
        self.state.select(None);
        self.update_input_fields();
    }

    pub fn update_input_fields(&mut self) {
        if let Some(selected) = self.state.selected() {
            let filtered = self.filtered_machines();
            let sorted = self.sorted_machines(filtered);
            if selected < sorted.len() {
                let machine = &sorted[selected];
                self.show_input_field = machine.is_active()
                    && (!machine.auth_user_in_user_owns || !machine.auth_user_in_root_owns);
                self.selected_machine_ip = machine.ip.clone();
            } else {
                self.show_input_field = false;
                self.selected_machine_ip = None;
            }
        } else {
            self.show_input_field = false;
            self.selected_machine_ip = None;
        }
    }

    pub fn enter_flag_input_mode(&mut self) {
        if self.show_input_field {
            self.input_mode = InputMode::Flag;
        }
    }
}

pub async fn fetch_all_machines(client: &Client, htb_api_key: &str) -> AppResult<Vec<Machine>> {
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

pub async fn fetch_machines(client: &Client, htb_api_key: &str, url: &str) -> AppResult<Root> {
    let res = client
        .get(url)
        .header("Authorization", format!("Bearer {}", htb_api_key))
        .send()
        .await?
        .json::<Root>()
        .await?;
    
    // Populate with IP because by default paginated does not have information about IP
    // Maybe i should do it in spawn function but then if machine is active before
    // We would not see it in active pane
    let mut res_with_ip = res;
    for machine in &mut res_with_ip.data {
        if machine.is_active() {
            match client.get(format!("{}/machine/profile/{}", HTB_API_URL, machine.id))
                .header("Authorization", format!("Bearer {}", htb_api_key))
                .send()
                .await
                {
                    Ok(response) => {
                        if let Ok(json) = response.json::<Value>().await {
                            if let Some(ip) = json.get("info").and_then(|info| info.get("ip")).and_then(Value::as_str) {
                                machine.ip = Some(ip.to_string());
                            }
                        }
                    },
                    Err(e) => {
                        eprintln!("Error fetching machine info for {}: {}", machine.id, e);
                    }
                }
        }
    }

    Ok(res_with_ip)
}

