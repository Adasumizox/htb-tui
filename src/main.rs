use std::{io, env};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::{
    app::{App, AppResult, fetch_all_machines, spawn_machine, submit_flag},
    event::{Event, EventHandler},
    handler::handle_key_events,
    tui::Tui,
};

pub mod app;
pub mod event;
pub mod handler;
pub mod tui;
pub mod ui;


#[tokio::main]
async fn main() ->AppResult<()> { 
    let htb_api_key = env::var("HTB_API_KEY")?;

    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    let event_sender = tui.events.sender.clone();
    let mut app = App::new(htb_api_key, event_sender);
    app.request_fetch_machines();

    while app.running {
        tui.draw(&mut app)?;
        match tui.events.next().await? {
            Event::Tick => {}
                //app.tick(),
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            Event::FetchMachines => {
                let client = app.client.clone();
                let htb_api_key = app.htb_api_key.clone();
                let sender = tui.events.sender.clone();
                tokio::spawn(async move {
                    let result = fetch_all_machines(&client, &htb_api_key, &sender).await
                        .map_err(|e| e.to_string());
                    match result {
                        Ok(()) => {
                            sender.send(Event::FetchMachinesResult(Ok((Vec::new(), Ok(()))))).unwrap();
                        }
                        Err(e) => {
                            sender.send(Event::FetchMachinesResult(Ok((Vec::new(), Err(e))))).unwrap();
                        }
                    }
                });
            }
            Event::FetchMachinesResult(result) => {
                app.handle_fetch_machines_result(result);
            }
            Event::SpawnMachine(machine_id) => {
                let client = app.client.clone();
                let htb_api_key = app.htb_api_key.clone();
                let sender = tui.events.sender.clone();
                tokio::spawn(async move {
                    let result = spawn_machine(&client, &htb_api_key, machine_id).await;
                    if result.is_ok() {
                        sender.send(Event::UpdateList).unwrap();
                    }
                    sender.send(Event::SpawnMachineResult(result)).unwrap();
                });
            }
            Event::SpawnMachineResult(result) => {
                app.handle_spawn_machine_result(result);
            }
            Event::SubmitFlag(machine_id, flag) => {
                    let client = app.client.clone();
                    let htb_api_key = app.htb_api_key.clone();
                    let sender = tui.events.sender.clone();
                    tokio::spawn(async move {
                        let result = submit_flag(&client, &htb_api_key, machine_id, &flag).await;
                        if result.is_ok() {
                            sender.send(Event::UpdateList).unwrap();
                        }
                        sender.send(Event::SubmitFlagResult(result)).unwrap();
                    });
            }
            Event::SubmitFlagResult(result) => {
                app.handle_submit_flag_result(result);
            }
            Event::UpdateList => {
                app.request_fetch_machines();
            }
            Event::UpdateInfoMessage(message) => {
                app.set_info_message(message);
            }
        }
    }

    tui.exit()?;
    Ok(())
}
