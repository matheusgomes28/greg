use crate::models::{Event, EventStore};

use std::io::{BufReader, Read};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use ical::parser::ical::component::IcalEvent;
use ical::IcalParser;


pub fn read_events<R: Read, T: TimeZone>(source: BufReader<R>, tz: &T) -> anyhow::Result<EventStore<T>> {
    let mut store = EventStore::<T>::default();

    let reader= IcalParser::new(source);
    for entry in reader {
        let calendar = entry?;
        for event in calendar.events {

            let mut start: DateTime<T>;
            let mut summary: String;

            for prop in &event.properties {
                match prop.name.as_str() {
                    "DTSTART" => start = parse_start(prop.value.as_ref().map(|s| s.as_str()), tz)?,
                    "SUMMARY" => summary = prop.value.clone().unwrap_or(String::from("")),
                    _ => todo!(),
                }

            }

            store.add(start, Event::<T>{
                title: summary,
                desc: summary,
                start,
                end: start,
            });
        }

    }

    Ok(store)
}

fn parse_start<T: TimeZone>(dt_str: Option<&str>, tz: &T) -> anyhow::Result<DateTime<T>> {
    if dt_str.is_none() {
        anyhow::bail!("invalid date str");
    }

    let  dt_str= dt_str.unwrap();

    if let Some(utc_str) = dt_str.strip_suffix('Z') {
        let utc_datetime= NaiveDateTime::parse_from_str(utc_str, "%Y%m%dT%H%M%S")?;
        return Ok(Utc.from_utc_datetime(&utc_datetime).with_timezone(&tz));
    }

    anyhow::bail!("to be done")
}


fn print_event(event: &IcalEvent) {
    for prop in &event.properties {
        println!("{}: {:?}", prop.name, prop.value);
    }
    println!();
}

