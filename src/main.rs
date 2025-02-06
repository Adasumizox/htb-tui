use std::{error::Error, io, env};
use serde::{Deserialize, Serialize};
use serde::ser::StdError;
use serde_json::Value;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
    DefaultTerminal, Frame,
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

#[derive(Debug, Default)]
pub struct App {
    counter: u8,
    exit: bool,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Left => self.decrement_counter(),
            KeyCode::Right => self.increment_counter(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn increment_counter(&mut self) {
        self.counter += 1;
    }
    
    fn decrement_counter(&mut self) {
        self.counter -= 1;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title: Line = Line::from(" Counter App ".bold());
        let instructions: Line = Line::from(vec![
            " Decrement ".into(),
            "<Left>".blue().bold(),
            " Increment ".into(),
            "<Right>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);
        let block: Block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);
        
        let counter_text: Text = Text::from(vec![Line::from(vec![
                "Value: ".into(),
                self.counter.to_string().yellow(),
        ])]);

        Paragraph::new(counter_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}

//async fn fetch_active_machines(api_token: &str) -> Result<Machine, Box<dyn Error>> {
//    let client = reqwest::Client::new();
//    let url = format!("{}/machine/list", HTB_API_URL);
//    let res = client
//        .get(url)
//        .header("Authorization", format!("Bearer {}", API_TOKEN))
//        .send()
//        .await?
//        .json::<Machine>()
//        .await?;

    //println!("{?:}", res);
//    Ok(res)
//}

fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>());
}

#[tokio::main]
async fn main() ->Result<(), Box<dyn std::error::Error>> {
    let HTB_API_KEY = env::var("HTB_API_KEY")?;

    let client = reqwest::Client::new();
    let url = format!("{}/machine/paginated?per_page=100", HTB_API_URL);
    let res = client
        .get(url)
        .header("Authorization", format!("Bearer {}", HTB_API_KEY))
        .send()
        .await?
        .json::<Root>()
        .await?;
    let active_machines: Vec<Machine> = res.data;
    println!("{:?}", active_machines);
    let url = format!("{}/machine/list/retired/paginated?per_page=100", HTB_API_URL);
    let mut res = client
        .get(url)
        .header("Authorization", format!("Bearer {}", HTB_API_KEY))
        .send()
        .await?
        .json::<Root>()
        .await?;
    let mut retired_machines: Vec<Machine> = res.data;
    while res.links.next.is_some()
    {
        let url = res.links.next.unwrap();
        res = client
            .get(url)
            .header("Authorization", format!("Bearer {}", HTB_API_KEY))
            .send()
            .await?
            .json::<Root>()
            .await?;
        retired_machines.append(&mut res.data);
    }
    println!("{:?}", retired_machines);
    //let mut terminal = ratatui::init();
    //let app_result = App::default().run(&mut terminal);
    //ratatui::restore();
    //app_result
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Style;

    #[test]
    fn render() {
        let app = App::default();
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 4));

        app.render(buf.area, &mut buf);

        let mut expected = Buffer::with_lines(vec![
            "┏━━━━━━━━━━━━━━━━━ Counter App ━━━━━━━━━━━━━━━━━━┓",
            "┃                    Value: 0                    ┃",
            "┃                                                ┃",
            "┗━ Decrement <Left> Increment <Right> Quit <Q> ━━┛",
        ]);
        let title_style = Style::new().bold();
        let counter_style = Style::new().yellow();
        let key_style = Style::new().blue().bold();
        expected.set_style(Rect::new(18, 0, 13, 1), title_style);
        expected.set_style(Rect::new(28, 1, 1, 1), counter_style);
        expected.set_style(Rect::new(13, 3, 6, 1), key_style);
        expected.set_style(Rect::new(30, 3, 7, 1), key_style);
        expected.set_style(Rect::new(43, 3, 4, 1), key_style);

        assert_eq!(buf, expected);
    }

    #[test]
    fn handle_key_event() -> io::Result<()> {
        let mut app = App::default();
        app.handle_key_event(KeyCode::Right.into());
        assert_eq!(app.counter, 1);

        app.handle_key_event(KeyCode::Left.into());
        assert_eq!(app.counter, 0);

        let mut app = App::default();
        app.handle_key_event(KeyCode::Char('q').into());
        assert!(app.exit);

        Ok(())
    }
}
