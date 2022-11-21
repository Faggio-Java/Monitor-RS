use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use std::{
    error::Error, io, thread, time::{Duration, Instant}, path::{Path, PathBuf},
};

use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Corner, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Gauge, Paragraph, Wrap, Tabs, List, ListItem, ListState},
    Frame, Terminal,
};

use sysinfo::{
    NetworkExt, NetworksExt, 
    ProcessExt, System, 
    SystemExt, CpuExt
};

use std::fs::read_dir;

struct StatefulList<T> {
    state: ListState,
    pid: Vec<T>,
}

impl<T> StatefulList<T> {
    fn with_items(pid: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            pid,
        }
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.pid.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.pid.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

struct App<'a> {
    mem_percentage: u32,
    mem_used: u32,
    mem_total: u32,
    cpu_percentage: u32,
    distro: String,
    host: String,
    proc: String,
    hdd1: String,
    hdd2: String,
    pub titles: Vec<&'a str>,
    pub index: usize,
    pid: StatefulList<String>,
    pnum: Vec<String>,
}

impl<'a> App<'a> {
    fn new() -> App<'a> {
        App {
            mem_percentage: 0,
            mem_used: 0,
            mem_total: 0,
            cpu_percentage: 0,
            distro: "String".to_owned(),
            host: "String".to_owned(),
            proc: "String".to_owned(),
            hdd1: "Hard Drive Not Found".to_owned(),
            hdd2: "Hard Drive Not Found".to_owned(),
            pid: StatefulList::with_items(vec![("proccesses havent been proccessed yet (3 seconds)".to_owned())]),
            pnum: vec![("0".to_owned())],
            titles: vec!["General", "Processes"],
            index: 0,
        }
    }

    pub fn nexttab(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }

    pub fn previoustab(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.titles.len() - 1;
        }
    }

    fn kill(&mut self) {
        let selector = self.pid.state.selected().expect("REASON").to_string();
        let i: i32 = selector.parse().unwrap();

    
        
        std::process::Command::new("kill")
            .arg(self.pnum.remove(i as usize))
            .spawn()
            .expect("Kill Failed!");
        self.taskmng(); 
        
    }

    fn on_tick(&mut self) {
        let mut sys = System::new_all();

        let meminfo = std::fs::read_to_string("/proc/meminfo").unwrap();
        let line_one = meminfo.lines().find(|f| f.contains("MemAvailable:")).unwrap();
         let pretty_one = line_one.replace("MemAvailable:", "").replace(' ', "").replace("kB", "");
        let line_two = meminfo.lines().find(|f| f.contains("MemTotal:")).unwrap();
         let pretty_two = line_two.replace("MemTotal:", "").replace(' ', "").replace("kB", "");
        let total_one: u32 = pretty_one.parse().expect("Oh nuu");
         let mem_temp = total_one / 1000;
        let total_two: u32 = pretty_two.parse().expect("Oh nuu");
         let mem_total = total_two / 1000;
          let mem_used = mem_total - mem_temp;
           let temp = mem_used * 100;
            let percentage = temp / mem_total;
        self.mem_percentage = percentage;
        self.mem_used = mem_used;
        self.mem_total = mem_total;
 
        sys.refresh_all();

        for cpu in sys.cpus() {
            let percentage2 = cpu.cpu_usage() as u32 / sys.cpus().len() as u32;
            self.cpu_percentage = percentage2;
        }

        let reader = std::fs::read_to_string("/etc/os-release");
        let binding = reader.expect("Failed Abruptly Uwu");
         let line = binding.lines().find(|f| f.contains("NAME=")).unwrap();
          let output = line.replace("NAME=", "");
        self.distro = output;

          let host = std::fs::read_to_string("/etc/hostname").unwrap();
        self.host = host.to_string();

        let reader = std::fs::read_to_string("/proc/stat");
        let binding = reader.expect("Failed Abruptly Uwu");
         let line = binding.lines().find(|f| f.contains("processes")).unwrap();
          let output2 = line.replace("processes ", "");
        self.proc = output2;

        let reader = std::fs::read_to_string("/proc/partitions");
        let binding = reader.expect("Failed Abruptly Uwu");
         let line1 = binding.lines().find(|f| f.contains("sda")).unwrap();
         let line2 = binding.lines().find(|f| f.contains("sdb")).unwrap();
          let sda: Vec<_> = line1.split_whitespace().collect();
          let sdb: Vec<_> = line2.split_whitespace().collect();
           let sda = format!("{} {} Kb", sda[3], sdb[2]);
           let sdb = format!("{} {} Kb", sdb[3], sdb[2]);
        self.hdd1 = sda;
        self.hdd2 = sdb;
    }

    fn taskmng(&mut self) {

        let paths = read_dir(&Path::new("/proc")).unwrap();
        
        let names =
        paths.filter_map(|entry| {
          entry.ok().and_then(|e|
            e.path().file_name()
            .and_then(|n| n.to_str().map(|s| String::from(s)))
          )
        }).collect::<Vec<String>>();
    
        let filt: Vec<&String> = names.iter().filter(|x| x.bytes().all(|c| c.is_ascii_digit())).collect();
        let mut pid = vec![];
        let mut pnum = vec![];
        for i in 0..filt.len() {
            let form = format!("/proc/{}/status", filt[i]);
            let reader = std::fs::read_to_string(form);
             let binding = reader.expect("Failed Abruptly Uwu");
              let line = binding.lines().find(|f| f.contains("Name:")).unwrap();
               let output = line.replace("Name:", "").replace("\t", "").replace("/0", "");
               let format = format!("{} {}", filt[i].to_owned(), output.to_owned());
               let format2 = format!("{}", filt[i].to_owned());
               pid.push(format.to_owned());
               pnum.push(format2.to_owned());
        }

        self.pid = StatefulList::with_items(pid);
        self.pnum = pnum;
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let tick_rate = Duration::from_millis(250);
    let mut taskrate = Duration::from_secs(3);
    let app = App::new();
    let res = run_app(&mut terminal, app, tick_rate, taskrate);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
    mut taskrate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    let mut task_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Tab => {if app.index == 1 {app.previoustab()} else {app.nexttab()}},
                    KeyCode::Down => app.pid.next(),
                    KeyCode::Up => app.pid.previous(),
                    KeyCode::Enter => app.kill(),
                    _ => {}
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
        if task_tick.elapsed() >= taskrate {
            app.taskmng();
            taskrate = Duration::from_secs(3600);
            task_tick = Instant::now();
        }
    }
}

fn first_tab<B: Backend>(f: &mut Frame<B>, app: &App, zone1: Rect, zone2: Rect, zone3: Rect) {
    let gauge = Gauge::default()
    .block(Block::default().title("Memory Usage").borders(Borders::ALL))
    .gauge_style(Style::default().fg(Color::Yellow))
    .percent(app.mem_percentage.try_into().unwrap());
f.render_widget(gauge, zone1);

let gauge = Gauge::default()
.block(Block::default().title("Cpu Usage").borders(Borders::ALL))
.gauge_style(Style::default().fg(Color::Yellow))
.percent(app.cpu_percentage.try_into().unwrap());
f.render_widget(gauge, zone2);

let meminfo = format!("Memory Usage: {}Mb / {}Mb", app.mem_used, app.mem_total);
let host = format!("Host: NotLfs");
let distro = format!("Distro: SuicidalSquirrel");
let process = format!("Running Processes: {}", app.proc);
let sda = format!("{}",app.hdd1);
let sdb = format!("{}",app.hdd2);
let text = vec![
    Spans::from(host),
    Spans::from(distro),
    Spans::from(process),
    Spans::from(meminfo),
    Spans::from("Disk Size"),
    Spans::from(sda),
    Spans::from(sdb),
];

let create_block = |title| {
    Block::default()
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::White).fg(Color::Black))
        .title(Span::styled(
            title,
            Style::default().add_modifier(Modifier::BOLD),
        ))
};

let paragraph = Paragraph::new(text.clone())
.style(Style::default().bg(Color::White).fg(Color::Black))
.block(create_block("Detailed Info"))
.alignment(Alignment::Left)
.wrap(Wrap { trim: true });
f.render_widget(paragraph, zone3);
}

fn second_tab<B: Backend>(f: &mut Frame<B>, app: &mut App, zone1: Rect, zone2: Rect, zone3: Rect) {
    let chunks = Layout::default()
    .direction(Direction::Vertical)
    .margin(2)
    .constraints(
        [
            Constraint::Percentage(13),
            Constraint::Percentage(100),
        ]
        .as_ref(),
    )
    .split(f.size());

        let pids: Vec<ListItem> = app
        .pid
        .pid
        .iter()
        .map(|i| {
            let log = Spans::from(vec![Span::raw(i)]);

            ListItem::new(vec![
                Spans::from("-".repeat(chunks[1].width as usize)),
                log,
            ])
        })
        .collect();

    let proc = List::new(pids)
        .block(Block::default().borders(Borders::ALL).title("pid   name"))
        .start_corner(Corner::BottomLeft)
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        );

    f.render_stateful_widget(proc, chunks[1], &mut app.pid.state);

}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let mut sys = System::new_all();

    let size = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Percentage(13),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ]
            .as_ref(),
        )
        .split(size);

    let titles = app
        .titles
        .iter()
        .map(|t| {
            let (first, rest) = t.split_at(1);
            Spans::from(vec![
                Span::styled(first, Style::default().fg(Color::Yellow)),
                Span::styled(rest, Style::default().fg(Color::Green)),
            ])
        })
        .collect();
        
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Tabs"))
        .select(app.index)
        .style(Style::default().fg(Color::Cyan))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Black),
        );
    f.render_widget(tabs, chunks[0]);

    let inner = match app.index {
        0 => first_tab(f, app, chunks[1], chunks[2], chunks[3]),
        1 => second_tab(f, app, chunks[1], chunks[2], chunks[3]),
        _ => unreachable!(),
    };
}
