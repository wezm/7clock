/*
 * TODO:
 *
 * - 24 hour option
 * - seconds option
 */

use time::{format_description::FormatItem, macros::format_description, OffsetDateTime};

const TWELVE_HOUR_HMS: &[FormatItem] = format_description!("[hour]:[minute]:[second]");
const TWELVE_HOUR_HM: &[FormatItem] = format_description!("[hour]:[minute]");
const TWENTY_FOUR_HOUR_HMS: &[FormatItem] = format_description!("[hour]:[minute]:[second]");
const TWENTY_FOUR_HOUR_HM: &[FormatItem] = format_description!("[hour]:[minute]:[second]");

fn main() {
    let now = OffsetDateTime::now_local().unwrap();
    let time_str = now.format(&TWELVE_HOUR_HM).unwrap();

    println!("{}", segmentify(&time_str));
}

fn segmentify(s: &str) -> String {
    s.chars()
        .map(|ch| {
            if ch.is_ascii_digit() {
                std::char::from_u32(0x1FBC0 + ch as u32).unwrap()
            } else {
                ch
            }
        })
        .collect()
}
