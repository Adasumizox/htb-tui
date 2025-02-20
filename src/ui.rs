use crate::app::App;

use ratatui::{
    layout::{Constraint, Layout, Rect, Position},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph, Clear},
    text::{Line, Span},
    Frame,
};

pub fn render(app: &mut App, frame: &mut Frame) {
    let chunks = 
        Layout::vertical([Constraint::Percentage(90), 
            Constraint::Percentage(10)])
            .split(frame.area());

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
                        machine.name,
                        machine.os,
                        machine.difficulty,
                        user_owns_symbol,
                        root_owns_symbol
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

    frame.render_stateful_widget(list, chunks[0], &mut app.state.clone());

    let info_paragraph = Paragraph::new(app.info_message.clone())
        .style(Style::default().fg(Color::LightCyan))
        .block(Block::default().borders(Borders::ALL).title("Info"));

    frame.render_widget(info_paragraph, chunks[1]);

    if app.show_input_field {
        if let Some(selected) = app.state.selected() {
            if selected < sorted_machines.len() {
                let machine = &sorted_machines[selected];
                let area = frame.area();
                let details_chunk =
                    Layout::horizontal([Constraint::Length(42), Constraint::Min(0)]).split(Rect::new(
                        area.width / 2 - 21,
                        area.height / 2 - 5,
                        80,
                        10,
                    ));

                let active_info = Paragraph::new(vec![
                    Line::from(vec![
                        Span::styled(
                            "Active machine: ",
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(&machine.name),
                    ]),
                    Line::from(vec![
                        Span::styled(
                            "IP Address: ",
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(app.selected_machine_ip.as_deref().unwrap_or("N/A")),
                    ]),
                ])
                .style(Style::default().fg(Color::White))
                .block(Block::default().borders(Borders::ALL).title("Active Machine Info"));

                frame.render_widget(Clear, details_chunk[0]);
                frame.render_widget(active_info, details_chunk[0]);

                let input_chunks =
                    Layout::vertical([Constraint::Length(3), Constraint::Length(3)]).split(details_chunk[1]);

                let flag_block = Paragraph::new(app.flag_input.clone())
                    .style(match app.input_mode {
                        crate::app::InputMode::Flag => Style::default().fg(Color::Yellow),
                        _ => Style::default().fg(Color::White),
                    })
                    .block(Block::default().borders(Borders::ALL).title("Flag"));

                frame.render_widget(Clear, input_chunks[0]);
                frame.render_widget(flag_block, input_chunks[0]);

                match app.input_mode {
                    crate::app::InputMode::Flag => {
                        frame.set_cursor_position(Position::new(
                            input_chunks[0].x + app.flag_input.len() as u16 + 1,
                            input_chunks[0].y + 1,
                        ));
                    }
                    _ => {}
                }
            }
        }
    }
}
