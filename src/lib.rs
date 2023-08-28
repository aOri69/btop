use crossterm::event::{self, Event, KeyCode};
use ratatui::{prelude::*, widgets::*};
use std::{
    io,
    time::{Duration, Instant},
};

mod app;
mod config;
pub use app::App;
pub use config::Config;

pub fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    // tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    let tick_rate = app.config.tick_rate();
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
    // main frame
    f.render_widget(Block::default().borders(Borders::ALL), f.size());

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Min(15),
                Constraint::Min(3),
                // Constraint::Percentage(50),
                Constraint::Min(20),
            ]
            .as_ref(),
        )
        .split(f.size());
    let title_area = layout[0];

    f.render_widget(
        Paragraph::new("Battery info. Press 'q' to quit")
            .dark_gray()
            .alignment(Alignment::Center),
        title_area,
    );

    let label = Span::styled(
        format!("{:.2}%", app.percentage * 100.0),
        Style::default()
            // .fg(Color::Green)
            .add_modifier(Modifier::SLOW_BLINK | Modifier::BOLD),
    );

    let gauge = Gauge::default()
        .block(Block::default().title("Percentage").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::White))
        .ratio(app.percentage)
        .label(label)
        .use_unicode(true);

    // f.render_widget(gauge.block(block), layout[2]);
    f.render_widget(gauge, layout[2]);

    let text = vec![
        Line::from(format!("Model - {}", app.model)),
        Line::from(format!("Serial - {}", app.serial_number)),
        Line::from(format!("Technology - {}", app.technology)),
        Line::from(format!("State - {}", app.state)),
        Line::from(format!("Health - {:.2}%", app.state_of_health * 100.0)),
        Line::from(format!("Cycles - {}", app.cycle_count)),
        Line::from(""),
        Line::from(format!(
            "Voltage - {:.3} Volts",
            app.voltage
                .get::<battery::units::electric_potential::volt>()
        )),
        Line::from(format!(
            "Power - {} W",
            app.power.get::<battery::units::power::watt>()
        )),
        Line::from(format!(
            "Energy - {} W/h",
            app.energy.get::<battery::units::energy::watt_hour>()
        )),
        Line::from(format!(
            "Time to empty - {:.2}h({:.2}m)",
            app.time_to_empty.get::<battery::units::time::hour>(),
            app.time_to_empty.get::<battery::units::time::minute>()
        )),
        Line::from(format!(
            "Time to full - {:.2}h({:.2}m)",
            app.time_to_full.get::<battery::units::time::hour>(),
            app.time_to_full.get::<battery::units::time::minute>()
        )),
        Line::from(format!(
            "Temperature - {}",
            app.temperature
                .get::<battery::units::thermodynamic_temperature::degree_celsius>()
        )),
    ];

    let paragraph = Paragraph::new(text).wrap(Wrap { trim: false });
    f.render_widget(
        paragraph.block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Battery - {}", &app.vendor)),
        ),
        layout[1],
    );

    if app.config.graph() {
        let x_axis: Vec<_> = (0..app.config.buf_capacity())
            .map(|i| (i as f64, 0.0))
            .collect();

        let graph_data = &app.get_power_bar_grid();
        let datasets = vec![
            Dataset::default()
                .name("Power draw")
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(Color::LightGreen))
                .graph_type(GraphType::Line)
                .data(graph_data),
            Dataset::default()
                .name("X axis")
                .marker(symbols::Marker::Dot)
                .dark_gray()
                .graph_type(GraphType::Line)
                .data(&x_axis),
        ];

        // Leftmost value of the graph buffer
        let left_label = app.power_bar.front().unwrap_or(&(0.0));
        // Average
        let mid_label = app.power_bar.iter().sum::<f64>() / app.power_bar.len() as f64;
        // Current value
        let right_label = app.power_bar.back().unwrap_or(&(0.0));

        let x_labels = vec![
            Span::styled(
                format!("{:.2}", left_label),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("AVG:{:.2}", mid_label)),
            Span::styled(
                format!("CUR:{:.2}", right_label),
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ];

        let upper_bound = app.power_bar_y_max + app.config.graph_clearance();
        let lower_bound = if app.power_bar_y_min > 0.0 {
            0.0
        } else {
            app.power_bar_y_min - app.config.graph_clearance()
        };
        let median = (upper_bound + lower_bound) / 2.0;

        let chart = Chart::new(datasets)
            .block(
                Block::default()
                    .title("Power".cyan().bold())
                    .borders(Borders::ALL),
            )
            .x_axis(
                Axis::default()
                    .title("Time(s)")
                    .style(Style::default().fg(Color::Gray))
                    .bounds([0.0, app.config.buf_capacity() as f64])
                    .labels(x_labels),
            )
            .y_axis(
                Axis::default()
                    .title("Power(W)")
                    .style(Style::default().fg(Color::Gray))
                    .bounds([lower_bound, upper_bound])
                    .labels(vec![
                        format!("{:.2}", lower_bound).into(),
                        format!("{:.2}", median).into(),
                        format!("{:.2}", upper_bound).into(),
                    ]),
            );

        f.render_widget(chart, layout[3]);
    }
}
