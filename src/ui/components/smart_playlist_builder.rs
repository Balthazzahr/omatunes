use iced::widget::{button, column, container, row, text, text_input, Space, checkbox, pick_list, scrollable};
use iced::{Alignment, Element, Length};
use crate::app::{Message, SmartPlaylistBuilderEvent};
use crate::ui::theme;
use crate::library::smart_playlist::{RuleField, RuleOperator, DateUnit, SmartPlaylistOrder, SmartPlaylistRule, SmartPlaylist};
use crate::ui::components::autocomplete::{get_suggestions, render_suggestions};

#[derive(Debug, Clone)]
pub struct RuleRowState {
    pub field: RuleField,
    pub operator: RuleOperator,
    pub value: String,
    pub value2: String,
    pub date_unit: DateUnit,
    pub boolean_value: bool,
}

#[derive(Debug, Clone)]
pub struct SmartPlaylistBuilderState {
    pub name: String,
    pub rules: Vec<RuleRowState>,
    pub limit_enabled: bool,
    pub limit_str: String,
    pub order_by: SmartPlaylistOrder,
    pub live_updating: bool,
    pub editing_name: Option<String>,
}

impl RuleRowState {
    pub fn new(field: RuleField) -> Self {
        let ops = valid_operators(field);
        let operator = ops.first().cloned().unwrap_or(RuleOperator::Is);
        RuleRowState {
            field,
            operator,
            value: String::new(),
            value2: String::new(),
            date_unit: DateUnit::Days,
            boolean_value: true,
        }
    }

    pub fn to_rule(&self) -> SmartPlaylistRule {
        let val = match self.field {
            RuleField::Liked => if self.boolean_value { "Liked".to_string() } else { "Not Liked".to_string() },
            RuleField::HasLyrics => if self.boolean_value { "Yes".to_string() } else { "No".to_string() },
            _ => self.value.clone(),
        };

        SmartPlaylistRule {
            field: self.field,
            operator: self.operator,
            value: val,
            value2: if self.operator == RuleOperator::Between { Some(self.value2.clone()) } else { None },
            date_unit: if self.field == RuleField::LastPlayed { Some(self.date_unit) } else { None },
        }
    }

    pub fn from_rule(rule: &SmartPlaylistRule) -> Self {
        let boolean_value = match rule.field {
            RuleField::Liked => rule.value.to_lowercase() == "liked",
            RuleField::HasLyrics => rule.value.to_lowercase() == "yes" || rule.value.to_lowercase() == "true",
            _ => false,
        };

        RuleRowState {
            field: rule.field,
            operator: rule.operator,
            value: rule.value.clone(),
            value2: rule.value2.clone().unwrap_or_default(),
            date_unit: rule.date_unit.unwrap_or(DateUnit::Days),
            boolean_value,
        }
    }
}

impl RuleField {
    pub fn all() -> &'static [RuleField] {
        &[
            RuleField::Title,
            RuleField::Artist,
            RuleField::Album,
            RuleField::Genre,
            RuleField::Year,
            RuleField::PlayCount,
            RuleField::Duration,
            RuleField::DiscNumber,
            RuleField::Liked,
            RuleField::HasLyrics,
            RuleField::LastPlayed,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            RuleField::Title => "Title",
            RuleField::Artist => "Artist",
            RuleField::Album => "Album",
            RuleField::Genre => "Genre",
            RuleField::Year => "Year",
            RuleField::PlayCount => "Play Count",
            RuleField::Duration => "Duration",
            RuleField::DiscNumber => "Disc Number",
            RuleField::Liked => "Liked",
            RuleField::HasLyrics => "Has Lyrics",
            RuleField::LastPlayed => "Last Played",
        }
    }
}

impl std::fmt::Display for RuleField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

impl RuleOperator {
    pub fn label(&self) -> &'static str {
        match self {
            RuleOperator::Contains => "contains",
            RuleOperator::DoesNotContain => "does not contain",
            RuleOperator::Is => "is",
            RuleOperator::IsNot => "is not",
            RuleOperator::Before => "is before",
            RuleOperator::After => "is after",
            RuleOperator::Between => "is between",
            RuleOperator::GreaterThan => "is greater than",
            RuleOperator::LessThan => "is less than",
            RuleOperator::WithinLast => "within the last",
        }
    }
}

impl std::fmt::Display for RuleOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

impl DateUnit {
    pub fn all() -> &'static [DateUnit] {
        &[DateUnit::Days, DateUnit::Weeks, DateUnit::Months]
    }

    pub fn label(&self) -> &'static str {
        match self {
            DateUnit::Days => "days",
            DateUnit::Weeks => "weeks",
            DateUnit::Months => "months",
        }
    }
}

impl std::fmt::Display for DateUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

impl SmartPlaylistOrder {
    pub fn all() -> &'static [SmartPlaylistOrder] {
        &[
            SmartPlaylistOrder::Random,
            SmartPlaylistOrder::MostPlayed,
            SmartPlaylistOrder::RecentlyPlayed,
            SmartPlaylistOrder::Year,
            SmartPlaylistOrder::Title,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            SmartPlaylistOrder::Random => "Random",
            SmartPlaylistOrder::MostPlayed => "Most Played",
            SmartPlaylistOrder::RecentlyPlayed => "Recently Played",
            SmartPlaylistOrder::Year => "Year",
            SmartPlaylistOrder::Title => "Title",
        }
    }
}

impl std::fmt::Display for SmartPlaylistOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

pub fn valid_operators(field: RuleField) -> Vec<RuleOperator> {
    match field {
        RuleField::Title | RuleField::Artist | RuleField::Album => vec![
            RuleOperator::Contains,
            RuleOperator::DoesNotContain,
            RuleOperator::Is,
            RuleOperator::IsNot,
        ],
        RuleField::Genre => vec![
            RuleOperator::Is,
            RuleOperator::IsNot,
        ],
        RuleField::Year => vec![
            RuleOperator::Is,
            RuleOperator::Before,
            RuleOperator::After,
            RuleOperator::Between,
        ],
        RuleField::PlayCount => vec![
            RuleOperator::Is,
            RuleOperator::GreaterThan,
            RuleOperator::LessThan,
        ],
        RuleField::Duration => vec![
            RuleOperator::GreaterThan,
            RuleOperator::LessThan,
        ],
        RuleField::DiscNumber => vec![
            RuleOperator::Is,
        ],
        RuleField::Liked | RuleField::HasLyrics => vec![
            RuleOperator::Is,
        ],
        RuleField::LastPlayed => vec![
            RuleOperator::WithinLast,
        ],
    }
}

pub fn view<'a>(
    state: &'a SmartPlaylistBuilderState,
    unique_artists: &[String],
    unique_albums: &[String],
    unique_genres: &[String],
) -> Element<'a, Message> {
    let title_text = text(if state.editing_name.is_some() { "Edit Smart Playlist" } else { "Create Smart Playlist" })
        .size(20)
        .font(crate::ui::icons::UI_FONT_BOLD)
        .color(theme::text());

    let name_field = column![
        text("Playlist Name")
            .size(12)
            .color(theme::subtext())
            .font(crate::ui::icons::UI_FONT_BOLD),
        text_input("Name", &state.name)
            .on_input(|s| Message::SmartPlaylistBuilderMsg(SmartPlaylistBuilderEvent::NameChanged(s)))
            .padding(8)
            .width(Length::Fill),
    ]
    .spacing(4);

    let mut rules_col = column![].spacing(12);

    for (idx, row_state) in state.rules.iter().enumerate() {
        let field_pick = pick_list(
            RuleField::all().to_vec(),
            Some(row_state.field),
            move |f| Message::SmartPlaylistBuilderMsg(SmartPlaylistBuilderEvent::UpdateRuleField(idx, f))
        )
        .width(Length::Fixed(120.0));

        let valid_ops = valid_operators(row_state.field);
        let op_pick = pick_list(
            valid_ops,
            Some(row_state.operator),
            move |o| Message::SmartPlaylistBuilderMsg(SmartPlaylistBuilderEvent::UpdateRuleOperator(idx, o))
        )
        .width(Length::Fixed(140.0));

        let mut value_container = column![].spacing(4).width(Length::Fill);

        match row_state.field {
            RuleField::Liked => {
                let label_fn = |b: bool| if b { "Liked" } else { "Not Liked" };
                
                let view_liked_pick: Element<'a, Message> = pick_list(
                    vec!["Liked", "Not Liked"],
                    Some(label_fn(row_state.boolean_value)),
                    move |s| Message::SmartPlaylistBuilderMsg(SmartPlaylistBuilderEvent::UpdateRuleBoolean(idx, s == "Liked"))
                )
                .width(Length::Fixed(120.0))
                .into();

                value_container = value_container.push(view_liked_pick);
            }
            RuleField::HasLyrics => {
                let view_lyrics_pick: Element<'a, Message> = pick_list(
                    vec!["Yes", "No"],
                    Some(if row_state.boolean_value { "Yes" } else { "No" }),
                    move |s| Message::SmartPlaylistBuilderMsg(SmartPlaylistBuilderEvent::UpdateRuleBoolean(idx, s == "Yes"))
                )
                .width(Length::Fixed(100.0))
                .into();

                value_container = value_container.push(view_lyrics_pick);
            }
            RuleField::LastPlayed => {
                let days_input = text_input("number", &row_state.value)
                    .on_input(move |s| Message::SmartPlaylistBuilderMsg(SmartPlaylistBuilderEvent::UpdateRuleValue(idx, s)))
                    .padding(6)
                    .width(Length::Fixed(60.0));

                let unit_pick = pick_list(
                    DateUnit::all().to_vec(),
                    Some(row_state.date_unit),
                    move |u| Message::SmartPlaylistBuilderMsg(SmartPlaylistBuilderEvent::UpdateRuleDateUnit(idx, u))
                )
                .width(Length::Fixed(100.0));

                let content = row![days_input, Space::with_width(6), unit_pick]
                    .align_y(Alignment::Center);

                value_container = value_container.push(content);
            }
            _ => {
                if row_state.operator == RuleOperator::Between {
                    let input1 = text_input("start", &row_state.value)
                        .on_input(move |s| Message::SmartPlaylistBuilderMsg(SmartPlaylistBuilderEvent::UpdateRuleValue(idx, s)))
                        .padding(6)
                        .width(Length::Fill);

                    let input2 = text_input("end", &row_state.value2)
                        .on_input(move |s| Message::SmartPlaylistBuilderMsg(SmartPlaylistBuilderEvent::UpdateRuleValue2(idx, s)))
                        .padding(6)
                        .width(Length::Fill);

                    let content = row![input1, text(" and ").size(12), input2]
                        .align_y(Alignment::Center)
                        .width(Length::Fill);

                    value_container = value_container.push(content);
                } else {
                    let placeholder = match row_state.field {
                        RuleField::Year => "e.g. 2004",
                        RuleField::PlayCount => "e.g. 10",
                        RuleField::Duration => "e.g. 3:30",
                        _ => "Value",
                    };

                    let text_field = text_input(placeholder, &row_state.value)
                        .on_input(move |s| Message::SmartPlaylistBuilderMsg(SmartPlaylistBuilderEvent::UpdateRuleValue(idx, s)))
                        .padding(6)
                        .width(Length::Fill);

                    value_container = value_container.push(text_field);

                    // Add autocomplete suggestions if text field matches Artist, Album, or Genre
                    let autocomplete_items = match row_state.field {
                        RuleField::Artist => Some(unique_artists),
                        RuleField::Album => Some(unique_albums),
                        RuleField::Genre => Some(unique_genres),
                        _ => None,
                    };

                    if let Some(items) = autocomplete_items {
                        let suggestions = get_suggestions(&row_state.value, items);
                        if !suggestions.is_empty() {
                            let rendered = render_suggestions(&suggestions, move |s| {
                                Message::SmartPlaylistBuilderMsg(SmartPlaylistBuilderEvent::UpdateRuleValue(idx, s))
                            });
                            value_container = value_container.push(rendered);
                        }
                    }
                }
            }
        }

        let remove_btn = button(
            text("\u{f00d}") // X icon
                .font(crate::ui::icons::NERD_FONT_MONO)
                .color(theme::red())
                .size(12)
        )
        .on_press(Message::SmartPlaylistBuilderMsg(SmartPlaylistBuilderEvent::RemoveRule(idx)))
        .style(iced::widget::button::text)
        .padding(4);

        let row_content = row![
            field_pick,
            Space::with_width(8),
            op_pick,
            Space::with_width(8),
            value_container,
            Space::with_width(8),
            remove_btn,
        ]
        .align_y(Alignment::Center)
        .width(Length::Fill);

        rules_col = rules_col.push(row_content);
    }

    let add_rule_btn = button(
        row![
            text("+ Add Rule")
                .size(12)
                .font(crate::ui::icons::UI_FONT_BOLD)
                .color(theme::accent())
        ]
    )
    .on_press(Message::SmartPlaylistBuilderMsg(SmartPlaylistBuilderEvent::AddRule))
    .style(theme::secondary_button)
    .padding([6, 12]);

    let options_title = text("Playlist Options")
        .size(14)
        .font(crate::ui::icons::UI_FONT_BOLD)
        .color(theme::text());

    let limit_chk = checkbox("Limit to", state.limit_enabled)
        .on_toggle(|b| Message::SmartPlaylistBuilderMsg(SmartPlaylistBuilderEvent::ToggleLimit(b)))
        .size(16);

    let limit_input = text_input("X", &state.limit_str)
        .on_input(|s| Message::SmartPlaylistBuilderMsg(SmartPlaylistBuilderEvent::LimitStrChanged(s)))
        .padding(4)
        .width(Length::Fixed(60.0));

    let order_pick = pick_list(
        SmartPlaylistOrder::all().to_vec(),
        Some(state.order_by),
        |o| Message::SmartPlaylistBuilderMsg(SmartPlaylistBuilderEvent::UpdateOrderBy(o))
    )
    .width(Length::Fixed(140.0));

    let limit_row = row![
        limit_chk,
        Space::with_width(8),
        limit_input,
        Space::with_width(8),
        text("songs selected by"),
        Space::with_width(8),
        order_pick,
    ]
    .align_y(Alignment::Center);

    let live_updating_chk = checkbox("Live updating", state.live_updating)
        .on_toggle(|b| Message::SmartPlaylistBuilderMsg(SmartPlaylistBuilderEvent::ToggleLive(b)))
        .size(16);

    let save_btn = button(
        text("Save")
            .size(13)
            .color(theme::base())
            .font(crate::ui::icons::UI_FONT_BOLD)
    )
    .on_press(Message::SmartPlaylistBuilderMsg(SmartPlaylistBuilderEvent::Save))
    .style(theme::accent_button)
    .padding([8, 16]);

    let cancel_btn = button(
        text("Cancel")
            .size(13)
            .color(theme::text())
    )
    .on_press(Message::SmartPlaylistBuilderMsg(SmartPlaylistBuilderEvent::Cancel))
    .style(theme::secondary_button)
    .padding([8, 16]);

    let action_row = row![
        Space::with_width(Length::Fill),
        cancel_btn,
        Space::with_width(12),
        save_btn,
    ]
    .align_y(Alignment::Center)
    .width(Length::Fill);

    let content = column![
        title_text,
        Space::with_height(16),
        name_field,
        Space::with_height(16),
        text("Rules (Match ALL of the following rules)")
            .size(12)
            .color(theme::subtext())
            .font(crate::ui::icons::UI_FONT_BOLD),
        Space::with_height(8),
        rules_col,
        Space::with_height(12),
        add_rule_btn,
        Space::with_height(20),
        options_title,
        Space::with_height(12),
        limit_row,
        Space::with_height(12),
        live_updating_chk,
        Space::with_height(24),
        action_row,
    ]
    .spacing(0)
    .width(Length::Fill);

    container(scrollable(content))
        .padding(24)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

// Map custom ToggleLimit message wrapper to compile cleanly
#[derive(Debug, Clone)]
pub enum SmartPlaylistBuilderEventEx {
    ToggleLimit(bool),
    LimitStrChanged(String),
}
