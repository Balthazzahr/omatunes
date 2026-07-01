use std::path::PathBuf;
use std::time::SystemTime;
use chrono::{Local, NaiveDateTime};
use serde::{Deserialize, Serialize};
use crate::library::models::Track;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RuleField {
    Title,
    Artist,
    Album,
    Genre,
    Year,
    PlayCount,
    Duration,
    DiscNumber,
    Liked,
    HasLyrics,
    LastPlayed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RuleOperator {
    Contains,
    DoesNotContain,
    Is,
    IsNot,
    Before,
    After,
    Between,
    GreaterThan,
    LessThan,
    WithinLast,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DateUnit {
    Days,
    Weeks,
    Months,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SmartPlaylistOrder {
    Random,
    MostPlayed,
    RecentlyPlayed,
    Year,
    Title,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SmartPlaylistRule {
    pub field: RuleField,
    pub operator: RuleOperator,
    pub value: String,
    pub value2: Option<String>,
    pub date_unit: Option<DateUnit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartPlaylist {
    pub name: String,
    pub rules: Vec<SmartPlaylistRule>,
    pub limit: Option<usize>,
    pub order_by: SmartPlaylistOrder,
    pub live_updating: bool,
    pub tracks: Vec<PathBuf>,
}

pub fn parse_duration(s: &str) -> Option<u64> {
    let s = s.trim();
    if s.contains(':') {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() == 2 {
            let mins = parts[0].parse::<u64>().ok()?;
            let secs = parts[1].parse::<u64>().ok()?;
            Some(mins * 60 + secs)
        } else {
            None
        }
    } else {
        s.parse::<u64>().ok()
    }
}

pub fn evaluate_rules(track: &Track, rules: &[SmartPlaylistRule], recently_played: &[(PathBuf, String)]) -> bool {
    if rules.is_empty() {
        return true;
    }

    for rule in rules {
        if !evaluate_single_rule(track, rule, recently_played) {
            return false;
        }
    }
    true
}

fn evaluate_single_rule(track: &Track, rule: &SmartPlaylistRule, recently_played: &[(PathBuf, String)]) -> bool {
    match rule.field {
        RuleField::Title => match_text(&track.title, rule),
        RuleField::Artist => match_text(&track.artist, rule),
        RuleField::Album => match_text(&track.album, rule),
        RuleField::Genre => match_text(&track.genre, rule),
        RuleField::Year => match_numeric_option(track.year, rule),
        RuleField::DiscNumber => match_numeric_option(track.disc_number, rule),
        RuleField::PlayCount => match_numeric(track.play_count, rule),
        RuleField::Duration => match_duration(track.duration.as_secs(), rule),
        RuleField::Liked => {
            let expect_liked = rule.value.to_lowercase() == "liked";
            let actual_liked = track.liked;
            match rule.operator {
                RuleOperator::Is => actual_liked == expect_liked,
                RuleOperator::IsNot => actual_liked != expect_liked,
                _ => false,
            }
        }
        RuleField::HasLyrics => {
            let has_lyrics = !track.lyrics.trim().is_empty();
            let expect_has = rule.value.to_lowercase() == "yes" || rule.value.to_lowercase() == "true";
            match rule.operator {
                RuleOperator::Is => has_lyrics == expect_has,
                RuleOperator::IsNot => has_lyrics != expect_has,
                _ => false,
            }
        }
        RuleField::LastPlayed => {
            let last_played_str = recently_played.iter()
                .find(|(p, _)| p == &track.path)
                .map(|(_, date)| date.clone());

            let last_played_dt = match last_played_str {
                Some(date_str) => {
                    match NaiveDateTime::parse_from_str(&date_str, "%Y-%m-%d %H:%M") {
                        Ok(dt) => dt,
                        Err(_) => return false,
                    }
                }
                None => return false, // never played
            };

            let now = Local::now().naive_local();
            let duration = now.signed_duration_since(last_played_dt);

            let val = match rule.value.parse::<i64>() {
                Ok(v) => v,
                Err(_) => return false,
            };

            let unit = rule.date_unit.unwrap_or(DateUnit::Days);
            let limit_hours = match unit {
                DateUnit::Days => val * 24,
                DateUnit::Weeks => val * 24 * 7,
                DateUnit::Months => val * 24 * 30, // rough approximation
            };

            match rule.operator {
                RuleOperator::WithinLast => duration.num_hours() <= limit_hours,
                _ => false,
            }
        }
    }
}

fn match_text(field_val: &str, rule: &SmartPlaylistRule) -> bool {
    let field_lower = field_val.to_lowercase();
    let query_lower = rule.value.to_lowercase();

    match rule.operator {
        RuleOperator::Contains => field_lower.contains(&query_lower),
        RuleOperator::DoesNotContain => !field_lower.contains(&query_lower),
        RuleOperator::Is => field_lower == query_lower,
        RuleOperator::IsNot => field_lower != query_lower,
        _ => false,
    }
}

fn match_numeric(field_val: u32, rule: &SmartPlaylistRule) -> bool {
    let val = match rule.value.parse::<u32>() {
        Ok(v) => v,
        Err(_) => return false,
    };

    match rule.operator {
        RuleOperator::Is => field_val == val,
        RuleOperator::IsNot => field_val != val,
        RuleOperator::GreaterThan => field_val > val,
        RuleOperator::LessThan => field_val < val,
        _ => false,
    }
}

fn match_numeric_option(field_val: Option<u32>, rule: &SmartPlaylistRule) -> bool {
    let val = match rule.value.parse::<u32>() {
        Ok(v) => v,
        Err(_) => return false,
    };

    match rule.operator {
        RuleOperator::Is => field_val == Some(val),
        RuleOperator::IsNot => field_val != Some(val),
        RuleOperator::Before => field_val.map(|y| y < val).unwrap_or(false),
        RuleOperator::After => field_val.map(|y| y > val).unwrap_or(false),
        RuleOperator::Between => {
            let val2 = match rule.value2.as_ref().and_then(|v| v.parse::<u32>().ok()) {
                Some(v) => v,
                None => return false,
            };
            field_val.map(|y| y >= val && y <= val2).unwrap_or(false)
        }
        _ => false,
    }
}

fn match_duration(duration_secs: u64, rule: &SmartPlaylistRule) -> bool {
    let val = match parse_duration(&rule.value) {
        Some(v) => v,
        None => return false,
    };

    match rule.operator {
        RuleOperator::GreaterThan => duration_secs > val,
        RuleOperator::LessThan => duration_secs < val,
        RuleOperator::Is => duration_secs == val,
        _ => false,
    }
}

pub fn sort_and_limit_tracks(
    tracks: &mut Vec<Track>,
    order: SmartPlaylistOrder,
    limit: Option<usize>,
    recently_played: &[(PathBuf, String)],
) {
    match order {
        SmartPlaylistOrder::Random => {
            use rand::seq::SliceRandom;
            let mut rng = rand::thread_rng();
            tracks.shuffle(&mut rng);
        }
        SmartPlaylistOrder::MostPlayed => {
            tracks.sort_by(|a, b| b.play_count.cmp(&a.play_count));
        }
        SmartPlaylistOrder::RecentlyPlayed => {
            tracks.sort_by(|a, b| {
                let a_pos = recently_played.iter().position(|(p, _)| p == &a.path).unwrap_or(usize::MAX);
                let b_pos = recently_played.iter().position(|(p, _)| p == &b.path).unwrap_or(usize::MAX);
                a_pos.cmp(&b_pos)
            });
        }
        SmartPlaylistOrder::Year => {
            tracks.sort_by(|a, b| b.year.unwrap_or(0).cmp(&a.year.unwrap_or(0)));
        }
        SmartPlaylistOrder::Title => {
            tracks.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
        }
    }

    if let Some(limit_val) = limit {
        if tracks.len() > limit_val {
            tracks.truncate(limit_val);
        }
    }
}
