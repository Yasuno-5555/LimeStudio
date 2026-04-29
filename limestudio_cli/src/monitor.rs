use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, Sparkline},
    Terminal,
};
use std::io;
use std::time::{Duration, Instant};

pub fn run_monitor() -> anyhow::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    res
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>) -> anyhow::Result<()> {
    let tick_rate = Duration::from_millis(50);
    let mut last_tick = Instant::now();
    let mut data = vec![0u64; 100];
    let mut phase: f64 = 0.0;

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let sparkline = Sparkline::default()
                .block(
                    Block::default()
                        .title(" LIME LIVE WAVEFORM (Monitor) ")
                        .borders(Borders::ALL),
                )
                .data(&data)
                .style(ratatui::style::Style::default().fg(ratatui::style::Color::Cyan));
            f.render_widget(sparkline, size);
        })?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    return Ok(());
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            // Update mock data
            data.remove(0);
            let val = ((phase.sin() + 1.0) * 5.0) as u64;
            data.push(val);
            phase += 0.3;
            last_tick = Instant::now();
        }
    }
}
