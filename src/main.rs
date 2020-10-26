use druid::{
    im::{Vector, HashMap},
    widget::{Button, CrossAxisAlignment, Flex, FlexParams, Label, List, Padding, Scroll, TextBox},
    AppLauncher, Data, Lens, PlatformError, UnitPoint, Widget, WidgetExt, WindowDesc,
};
use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Data)]
struct TrackerEntry {
    entry_date: Arc<u128>,
    time_spent_ms: Arc<u128>,
    description: String,
}

#[derive(Clone, Data)]
struct TrackerSection {
    time_spent_ms: Arc<u128>,
    description: String,
}

#[derive(Clone, Data, Lens)]
struct TrackerInfo {
    tracker_start: Arc<u128>,
    description: String,
}

#[derive(Clone, Data, Lens)]
struct TrackerEntryData {
    storage: Vector<TrackerEntry>,
    section_storage: Vector<TrackerSection>,
    current_entry: Option<TrackerInfo>,
    incoming_descriptor: String,
}

impl TrackerEntryData {
    fn end_entry_calculation(&mut self, tracker_info: &TrackerInfo) {
        if tracker_info.description.is_empty() {
            self.current_entry = Option::None;
            return;
        }
        let time = get_current_ms();
        let time_spent = time - *tracker_info.tracker_start.clone();
        if time_spent == 0 {
            self.current_entry = Option::None;
            return;
        }
        self.storage.push_back(TrackerEntry {
            entry_date: Arc::new(time),
            time_spent_ms: Arc::new(time_spent),
            description: tracker_info.description.clone(),
        });
        self.current_entry = Option::None;
        self.parse_tracker_sections();
    }

    fn parse_tracker_sections(&mut self) {
        let mut section_map = HashMap::<String, u128>::new();

        for tracker_entry in &self.storage {
            let descriptor = tracker_entry.description.clone();
            let initial = *section_map.get(&descriptor).unwrap_or(&0u128);
            let incrementer = *tracker_entry.time_spent_ms.clone();
            section_map.insert(descriptor, initial + incrementer);
        }

        let mut new_storage = Vector::new();

        for entry in section_map {
            new_storage.push_back(TrackerSection {
                time_spent_ms: Arc::new(entry.1),
                description: entry.0,
            });
        }

        self.section_storage = new_storage;
        self.section_storage.sort_by(|o1, o2| o1.description.cmp(&o2.description))
    }

    fn start_entry_calculation(&mut self) {
        if self.incoming_descriptor.is_empty() {
            return;
        }
        self.current_entry = Option::Some(TrackerInfo {
            tracker_start: Arc::new(get_current_ms()),
            description: self.incoming_descriptor.clone(),
        })
    }

    fn swap_states(&mut self) {
        let current_info = {
            &self.current_entry.clone()
        };

        match current_info {
            Some(tracker_info) => {
                &self.end_entry_calculation(tracker_info);
            }
            None => {
                &self.start_entry_calculation();
            }
        };
    }
}

fn build_ui() -> impl Widget<TrackerEntryData> {
    let description_text_box = TextBox::new()
        .lens(TrackerEntryData::incoming_descriptor)
        .expand_width();

    let switch_button = Button::new(|data: &Option<TrackerInfo>, _env: &_| {
        match data {
            Some(_) => String::from("Stop"),
            None => String::from("Start")
        }
    }).lens(TrackerEntryData::current_entry).on_click(move |_event, data: &mut TrackerEntryData, _env| {
        data.swap_states();
    });

    let output_text = Label::new(|data: &Option<TrackerInfo>, _env: &_| {
        match data {
            Some(tracker_info) => {
                format!("Tracker Running: `{}`", &tracker_info.description)
            }
            None => {
                String::from("Tracker Not Started")
            }
        }
    }).lens(TrackerEntryData::current_entry);

    let scroll_list = Scroll::new(List::new(|| {
        Label::new(|item: &TrackerSection, _env: &_| {
            format!("| {} | {} |", item.description, ms_to_time(*item.time_spent_ms))
        }).align_vertical(UnitPoint::LEFT).padding(1.0).expand_width()
    })).vertical().lens(TrackerEntryData::section_storage);

    Padding::new(3.0, Flex::column()
        .with_flex_child(
            Flex::row()
                .with_flex_child(description_text_box, 1.0)
                .with_flex_spacer(0.1)
                .with_flex_child(switch_button, 1.0)
                .with_flex_spacer(0.1)
                .with_flex_child(output_text, 1.0),
            1.0)
        .with_flex_spacer(0.2)
        .with_flex_child(
            Label::new("| Description | Time Spent |"),
            FlexParams::new(1.0, CrossAxisAlignment::Start))
        .with_flex_spacer(0.05)
        .with_flex_child(scroll_list, 1.0),
    )
}

fn main() -> Result<(), PlatformError> {
    AppLauncher::with_window(WindowDesc::new(build_ui)).launch(TrackerEntryData {
        storage: Vector::new(),
        section_storage: Vector::new(),
        current_entry: Option::None,
        incoming_descriptor: String::from(""),
    })?;

    Ok(())
}

fn get_current_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time is not working properly.")
        .as_millis()
}

pub fn ms_to_time(time: u128) -> String {
    let mut seconds = time / 1000;
    let mut minutes = seconds / 60;
    seconds %= 60;
    let hours = minutes / 60;
    minutes %= 60;

    let mut display = String::new();
    let mut flag = false;

    if hours > 0 {
        display += format!("{} Hour", hours).as_str();
        if hours > 1 {
            display += "s";
        }
        flag = true;
    }

    if minutes > 0 {
        if flag {
            display += " ";
        }
        display += format!("{} Minute", minutes).as_str();
        if minutes > 1 {
            display += "s";
        }
        flag = true;
    }

    if seconds > 0 {
        if flag {
            display += " ";
        }
        display += format!("{} Second", seconds).as_str();
        if seconds > 1 {
            display += "s";
        }
    }

    display
}