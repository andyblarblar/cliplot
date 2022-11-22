//! iced based gui interface

use crate::extractor::Data;
use chrono::{DateTime, Utc};
use iced::{
    alignment::{Horizontal, Vertical},
    executor,
    widget::{
        canvas::{Cache, Frame, Geometry},
        Column, Container,
    },
    Alignment, Application, Command, Element, Font, Length, Size, Subscription, Theme,
};
use plotters::prelude::ChartBuilder;
use plotters_iced::plotters_backend::DrawingBackend;
use plotters_iced::{Chart, ChartWidget};
use std::collections::VecDeque;
use std::default::Default;
use std::time::Duration;

const FONT_REGULAR: Font = Font::External {
    name: "sans-serif-regular",
    bytes: include_bytes!("../fonts/notosans-regular.ttf"),
};

const FONT_BOLD: Font = Font::External {
    name: "sans-serif-bold",
    bytes: include_bytes!("../fonts/notosans-bold.ttf"),
};

#[derive(Debug)]
pub enum Message {
    /// message to update charts
    Tick,
    /// Data from stdin
    Data(Data),
}

#[derive(Default)]
pub struct Flags {
    pub num_channels: usize,
}

/// Application state
pub struct State {
    chart: SignalChart,
}

impl Application for State {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = Flags;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                chart: SignalChart::new(flags.num_channels),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "cliplot".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Tick => {} //TODO this actually prob isnt needed, just update whenever we get new data
            Message::Data(data) => self.chart.push_data(data),
        }
        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let content = Column::new()
            .spacing(20)
            .align_items(Alignment::Start)
            .width(Length::Fill)
            .height(Length::Fill)
            .push(self.chart.view());

        Container::new(content)
            //.style(style::Container)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(5)
            .center_x()
            .center_y()
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        const FPS: u64 = 60;

        // TODO add our runtime here
        Subscription::batch(
            [iced::time::every(Duration::from_millis(1000 / FPS)).map(|_| Message::Tick)]
                .into_iter(),
        )
    }
}

/// Widget that displays our chart
struct SignalChart {
    cache: Cache,
    /// Vector of signal channels. Channel numbers are indices
    data_points: Vec<VecDeque<(DateTime<Utc>, Data)>>,
    /// Size of the time domain we display
    plot_seconds: usize, //TODO make scalable with interface
    // Lazy track plotting info
    latest_reading: DateTime<Utc>,
    highest_reading: f64,
    lowest_reading: f64,
}

impl SignalChart {
    fn new(num_channels: usize) -> Self {
        let data_points = vec![VecDeque::new(); num_channels];
        Self {
            cache: Cache::new(),
            data_points,
            latest_reading: Default::default(),
            highest_reading: 1.0,
            lowest_reading: -1.0,
            plot_seconds: 5,
        }
    }

    /// Pushes data into its appropriate queue, then trims the old data.
    fn push_data(&mut self, value: Data) {
        let time = value.stamp;
        let cur_ms = time.timestamp_millis();
        let limit = Duration::from_secs(self.plot_seconds as u64);

        self.data_points[value.channel].push_front((time, value));
        // Trim old data
        loop {
            if let Some((time, _)) = self.data_points[value.channel].back() {
                let diff = Duration::from_millis((cur_ms - time.timestamp_millis()) as u64);
                if diff > limit {
                    self.data_points[value.channel].pop_back();
                    continue;
                }
            }
            break;
        }
        self.cache.clear();

        // If this reading is newer than any other, mark that as our latest
        if value.stamp > self.latest_reading {
            self.latest_reading = value.stamp;
        }

        // Update bounds
        if value.data > self.highest_reading {
            self.highest_reading = value.data
        } else if value.data < self.lowest_reading {
            self.lowest_reading = value.data
        }
    }

    fn view(&self) -> Element<Message> {
        Container::new(
            Column::new()
                .width(Length::Fill)
                .height(Length::Fill)
                .spacing(5)
                .push(
                    ChartWidget::new(self).height(Length::Fill).resolve_font(
                        |_, style| match style {
                            plotters_iced::plotters_backend::FontStyle::Bold => FONT_BOLD,
                            _ => FONT_REGULAR,
                        },
                    ),
                ),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .into()
    }
}

impl Chart<Message> for SignalChart {
    type State = ();

    fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, mut chart: ChartBuilder<DB>) {
        use plotters::{prelude::*, style::Color};

        const PLOT_LINE_COLOR: RGBColor = RGBColor(0, 175, 255);

        // Dynamically size the y axis as data comes in, then plot all data in the selected time domain
        let oldest_time = self.latest_reading - chrono::Duration::seconds(self.plot_seconds as i64);
        let mut chart = chart
            .x_label_area_size(28)
            .y_label_area_size(28)
            .margin(20)
            .build_cartesian_2d(
                oldest_time.timestamp_millis()..self.latest_reading.timestamp_millis(),
                self.lowest_reading..self.highest_reading,
            )
            .expect("failed to build chart");

        chart
            .configure_mesh()
            .bold_line_style(BLUE.mix(0.2))
            .light_line_style(BLUE.mix(0.1))
            .axis_style(ShapeStyle::from(BLUE.mix(0.80)).stroke_width(1))
            .y_labels(10)
            .y_label_style(
                ("sans-serif", 15)
                    .into_font()
                    .color(&BLUE.mix(0.80))
                    .transform(FontTransform::Rotate90),
            )
            .x_labels(10)
            .x_label_style(
                ("sans-serif", 15)
                    .into_font()
                    .color(&BLUE.mix(0.80))
                    .transform(FontTransform::Rotate90),
            )
            .draw()
            .expect("failed to draw chart mesh");

        // Plot each channel
        for channel in self.data_points.iter() {
            if !channel.is_empty() {
                chart
                    .draw_series(LineSeries::new(
                        channel.iter().map(|x| (x.0.timestamp_millis(), x.1.data)),
                        PLOT_LINE_COLOR,
                    ))
                    .expect("failed to draw chart data");
            }
        }
    }

    #[inline]
    fn draw<F: Fn(&mut Frame)>(&self, bounds: Size, draw_fn: F) -> Geometry {
        self.cache.draw(bounds, draw_fn)
    }
}
