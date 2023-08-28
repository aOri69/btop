use std::collections::VecDeque;

use crate::Config;

#[derive(Default)]
pub struct App {
    pub percentage: f64,
    pub vendor: String,
    pub power: battery::units::Power,
    pub energy: battery::units::Energy,
    pub time_to_empty: battery::units::Time,
    pub time_to_full: battery::units::Time,
    pub temperature: battery::units::ThermodynamicTemperature,
    pub voltage: battery::units::ElectricPotential,
    pub cycle_count: u32,
    pub model: String,
    pub serial_number: String,
    pub state: battery::State,
    pub state_of_health: f32,
    pub technology: battery::Technology,
    pub power_bar: VecDeque<f64>,
    pub power_bar_y_max: f64,
    pub power_bar_y_min: f64,
    pub config: Config,
}

impl App {
    pub fn new(config: Config) -> App {
        App {
            percentage: f64::default(),
            vendor: String::new(),
            power: battery::units::Power::default(),
            energy: battery::units::Energy::default(),
            time_to_empty: battery::units::Time::default(),
            time_to_full: battery::units::Time::default(),
            cycle_count: u32::default(),
            model: Default::default(),
            serial_number: Default::default(),
            state_of_health: Default::default(),
            state: Default::default(),
            technology: Default::default(),
            temperature: Default::default(),
            voltage: Default::default(),
            power_bar: VecDeque::with_capacity(config.buf_capacity()),
            power_bar_y_max: 0.1,
            power_bar_y_min: -0.1,
            config,
        }
    }

    pub fn on_tick(&mut self) {
        let manager = battery::Manager::new().expect("Expected battery manager");
        for (_idx, maybe_battery) in manager
            .batteries()
            .expect("Expected at least one battery")
            .take(1)
            .enumerate()
        {
            let battery = maybe_battery.expect("Expected battery");
            self.percentage = battery
                .state_of_charge()
                .get::<battery::units::ratio::percent>() as f64
                / 100.0;
            self.vendor = battery.vendor().unwrap_or_default().to_owned();
            self.power = battery.energy_rate();
            self.energy = battery.energy();
            self.time_to_empty = battery.time_to_empty().unwrap_or_default();
            self.time_to_full = battery.time_to_full().unwrap_or_default();
            self.temperature = battery.temperature().unwrap_or_default();
            self.voltage = battery.voltage();
            self.cycle_count = battery.cycle_count().unwrap_or_default();
            self.model = battery.model().unwrap_or_default().to_owned();
            self.serial_number = battery.serial_number().unwrap_or("Unknown").to_owned();
            self.state = battery.state();
            self.state_of_health = battery
                .state_of_health()
                .get::<battery::units::ratio::percent>()
                / 100.0;
            self.technology = battery.technology();

            let signed_power = match self.state {
                battery::State::Charging => {
                    self.power.get::<battery::units::power::watt>() as f64 * -1.0
                }
                _ => self.power.get::<battery::units::power::watt>() as f64,
            };
            let upper_index = self.config.upper_index();
            match self.power_bar.len() {
                // runtime values cannot be referenced in patterns
                // _l @ 0..=upper_index => self.power_bar.push_back(signed_power),
                l if l <= upper_index => self.power_bar.push_back(signed_power),
                _ => {
                    self.power_bar.pop_front();
                    self.power_bar.push_back(signed_power);
                }
            }

            self.power_bar_y_max = self
                .power_bar
                .iter()
                .max_by(|&l, &r| l.partial_cmp(r).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or(&f64::default())
                .to_owned();
            self.power_bar_y_min = self
                .power_bar
                .iter()
                .min_by(|&l, &r| l.partial_cmp(r).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or(&f64::default())
                .to_owned();
        }
    }

    pub fn get_power_bar_grid(&self) -> Vec<(f64, f64)> {
        self.power_bar
            .iter()
            .rev()
            .enumerate()
            .map(|(idx, &y)| {
                let x = self.config.upper_index() - idx;
                (x as f64, y)
            })
            .collect()
    }
}
