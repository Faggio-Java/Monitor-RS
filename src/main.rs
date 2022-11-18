use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use std::{
    error::Error, io, thread, time::{Duration, Instant},
};

use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Gauge, Paragraph, Wrap},
    Frame, Terminal,
};

use sysinfo::{
    NetworkExt, NetworksExt, 
    ProcessExt, System, 
    SystemExt, CpuExt
};

struct App {
    mem_percentage: u32,
    mem_used: u32,
    mem_total: u32,
    cpu_percentage: u32,
    distro: String,
    host: String,
    proc: String,
}

impl App {
    fn new() -> App {
        App {
            mem_percentage: 0,
            mem_used: 0,
            mem_total: 0,
            cpu_percentage: 0,
            distro: "String".to_owned(),
            host: "String".to_owned(),
            proc: "String".to_owned(),
        }
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
            std::thread::sleep(std::time::Duration::from_millis(300));
        }

        let reader = std::fs::read_to_string("/etc/os-release");
        let binding = reader.expect("REASON");
         let line = binding.lines().find(|f| f.contains("NAME=")).unwrap();
          let output = line.replace("NAME=", "");
        self.distro = output;

          let host = std::fs::read_to_string("/etc/hostname").unwrap();
        self.host = host.to_string();

        let reader = std::fs::read_to_string("/proc/stat");
        let binding = reader.expect("REASON");
         let line = binding.lines().find(|f| f.contains("processes")).unwrap();
          let output2 = line.replace("processes ", "");
        self.proc = output2;
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
    let app = App::new();
    let res = run_app(&mut terminal, app, tick_rate);

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
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    return Ok(());
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let mut sys = System::new_all();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ]
            .as_ref(),
        )
        .split(f.size());
        let meminfo = format!("Memory Usage: {}Mb / {}Mb", app.mem_used, app.mem_total);
        let host = format!("Host: {}", app.host);
        let distro = format!("Distro: {}", app.distro);
        let process = format!("Running Processes: {}", app.proc);
        let text = vec![
            Spans::from(host),
            Spans::from(distro),
            Spans::from(process),
            Spans::from(meminfo),
        ];

    let gauge = Gauge::default()
        .block(Block::default().title("Memory Usage").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Yellow))
        .percent(app.mem_percentage.try_into().unwrap());
    f.render_widget(gauge, chunks[0]);

    let gauge = Gauge::default()
    .block(Block::default().title("Cpu Usage").borders(Borders::ALL))
    .gauge_style(Style::default().fg(Color::Yellow))
    .percent(app.cpu_percentage.try_into().unwrap());
f.render_widget(gauge, chunks[1]);


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
f.render_widget(paragraph, chunks[2]);
}
