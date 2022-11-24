//! iced based gui interface

use crate::extractor::{extract_channels, Config, Data};
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
use std::sync::Arc;
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
    /// Data from stdin
    Data(Vec<Data>),
    /// Stdin was closed
    Closed,
}

#[derive(Default)]
pub struct Flags {
    pub extractor_conf: Arc<Config>,
}

/// Application state
pub struct State {
    chart: SignalChart,
    extractor_conf: Arc<Config>,
    stdin_closed: bool,
}

impl Application for State {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = Flags;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                chart: SignalChart::new(
                    flags.extractor_conf.matchers.len(),
                    Utc::now().timestamp_millis(),
                ),
                extractor_conf: flags.extractor_conf,
                stdin_closed: false,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "cliplot".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Data(data) => data.into_iter().for_each(|d| self.chart.push_data(d)),
            Message::Closed => self.stdin_closed = true,
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
        // Keep reading until stdin closes, then avoid freezing gui
        if self.stdin_closed {
            Subscription::none()
        } else {
            extract_channels(self.extractor_conf.clone())
        }
    }
}

/// Widget that displays our chart
struct SignalChart {
    cache: Cache,
    /// Vector of signal channels. Channel numbers are indices
    data_points: Vec<VecDeque<(DateTime<Utc>, Data)>>,
    /// Size of the time domain we display
    plot_ms: u64, //TODO make scalable with interface
    /// Start time of graphing in unix epoch
    start_time_ms: i64,
    // Lazy track plotting info
    latest_reading: DateTime<Utc>,
    highest_reading: f64,
    lowest_reading: f64,
}

impl SignalChart {
    fn new(num_channels: usize, start_time_ms: i64) -> Self {
        let data_points = vec![VecDeque::new(); num_channels];
        Self {
            cache: Cache::new(),
            data_points,
            latest_reading: chrono::DateTime::default(),
            highest_reading: 1.0,
            lowest_reading: 0.0,
            plot_ms: 5000,
            start_time_ms,
        }
    }

    /// Pushes data into its appropriate queue, then trims the old data.
    fn push_data(&mut self, value: Data) {
        let cur_ms = value.stamp.timestamp_millis();
        let limit = Duration::from_millis(self.plot_ms);

        self.data_points[value.channel].push_front((value.stamp, value));
        // Trim data if it is older than the timespan shown on the graph
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
            self.highest_reading = value.data;
        } else if value.data < self.lowest_reading {
            self.lowest_reading = value.data;
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

        const PLOT_LINE_COLOR1: RGBColor = RGBColor(0, 175, 255);
        const PLOT_LINE_COLOR2: RGBColor = RGBColor(175, 0, 255);

        // Dynamically size the y axis as data comes in, then plot all data in the selected time domain
        let oldest_time = self.latest_reading - chrono::Duration::milliseconds(self.plot_ms as i64);
        let mut chart = chart
            .x_label_area_size(28)
            .y_label_area_size(28)
            .margin(20)
            .build_cartesian_2d(
                oldest_time.timestamp_millis() - self.start_time_ms
                    ..self.latest_reading.timestamp_millis() - self.start_time_ms,
                self.lowest_reading..self.highest_reading,
            )
            .expect("failed to build chart");

        chart
            .configure_mesh()
            .bold_line_style(BLUE.mix(0.4))
            .light_line_style(BLUE.mix(0.2))
            .axis_style(ShapeStyle::from(BLUE.mix(0.80)).stroke_width(1))
            .y_labels(10)
            .y_label_style(
                ("sans-serif", 15)
                    .into_font()
                    .color(&BLUE.mix(0.80))
                    .transform(FontTransform::Rotate90),
            )
            .x_label_formatter(&|d| format!("{}ms", d))
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
        for (i, channel) in self.data_points.iter().enumerate() {
            if !channel.is_empty() {
                chart
                    .draw_series(LineSeries::new(
                        channel
                            .iter()
                            .map(|x| (x.0.timestamp_millis() - self.start_time_ms, x.1.data)),
                        if i % 2 == 0 {
                            PLOT_LINE_COLOR1
                        } else {
                            PLOT_LINE_COLOR2
                        }, //TODO change per channel
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
