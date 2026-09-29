use chrono::{Datelike, Days, Duration, NaiveDate, NaiveDateTime, NaiveTime};
use chrono_tz::Europe::Bucharest;
use clap::Parser;
use icalendar::{Calendar, Component, Event, EventLike, RRule, Tz};
use serde::Deserialize;
use std::{collections::HashSet, fs};

use crate::Periodicity::EvenWeekly;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    config: String,
}

#[derive(Debug, Deserialize)]
struct Semester {
    name: String,
    start_date: String,
    end_date: String,
    free_days: Vec<String>, // format: YYYY-MM-DD or YYYY-MM-DD->YYYY-MM-DD
}
#[derive(Debug, Deserialize, PartialEq, Eq)]
enum Periodicity {
    Weekly,
    OddWeekly,
    EvenWeekly,
}

#[derive(Debug, Deserialize)]
struct Instance {
    start_time: String,
    end_time: String,
    day_of_week: u64, // starts from 1
    periodicity: Periodicity,
}

#[derive(Debug, Deserialize)]
enum ActivityType {
    Course,
    Lab,
    Seminar,
}

impl ToString for ActivityType {
    fn to_string(&self) -> std::string::String {
        format!(
            "{}",
            match self {
                ActivityType::Course => "Curs",
                ActivityType::Lab => "Laborator",
                ActivityType::Seminar => "Seminar",
            }
        )
    }
}

impl Activity {
    pub fn cal_name(&self) -> String {
        format!("{} - {}", self.activity_type.to_string(), self.name)
    }

    pub fn description(&self) -> String {
        format!("Profesor: {}", self.professor)
    }
}

#[derive(Debug, Deserialize)]
struct Activity {
    name: String,
    #[serde(rename = "type")]
    activity_type: ActivityType,
    location: String,
    professor: String,
    instances: Vec<Instance>,
}

#[derive(Debug, Deserialize)]
struct Config {
    activities: Vec<Activity>,
    semester: Semester,
}

fn main() {
    let args = Args::parse();

    let contents =
        fs::read_to_string(args.config).expect("Should have been able to read the config");

    let config = yaml_serde::from_str::<Config>(&contents).expect("Can't parse the yaml");

    println!("Total activities: {}", config.activities.len());

    let sem_start = NaiveDate::parse_from_str(&config.semester.start_date, "%Y-%m-%d")
        .expect("wrong format for sem start date");

    let sem_end: chrono::prelude::DateTime<Tz> =
        NaiveDate::parse_from_str(&config.semester.end_date, "%Y-%m-%d")
            .expect("wrong format for sem end date")
            .and_time(NaiveTime::MIN)
            .and_local_timezone(icalendar::Tz::UTC)
            .single()
            .unwrap();

    let mut calendar = Calendar::new();

    let mut exdates: HashSet<NaiveDate> = HashSet::new();

    for free_day in &config.semester.free_days {
        let parts: Vec<&str> = free_day.split("->").collect();

        let (s, e) = if parts.len() == 2 {
            (
                NaiveDate::parse_from_str(parts[0].trim(), "%Y-%m-%d"),
                NaiveDate::parse_from_str(parts[1].trim(), "%Y-%m-%d"),
            )
        } else {
            (
                NaiveDate::parse_from_str(free_day.trim(), "%Y-%m-%d"),
                NaiveDate::parse_from_str(free_day.trim(), "%Y-%m-%d"),
            )
        };

        let start_date = s.expect("Wrong date format for start date");
        let end_date = e.expect("Wrong date format for end date");

        let mut i_date = start_date;
        while i_date <= end_date {
            exdates.insert(i_date);
            i_date += Duration::days(1);
        }
    }

    for activity in config.activities {
        for f_instance in &activity.instances {
            let mut event = Event::new();
            event.location(&activity.location);
            event.summary(&activity.cal_name());
            event.description(&activity.description());

            let start_time = NaiveTime::parse_from_str(&f_instance.start_time, "%H:%M")
                .expect("Wrong start time format");
            let end_time = NaiveTime::parse_from_str(&f_instance.end_time, "%H:%M")
                .expect("Wrong end time format");

            let offs = if f_instance.periodicity == EvenWeekly {
                7
            } else {
                0
            };

            let start_date = NaiveDateTime::new(
                sem_start + Days::new(f_instance.day_of_week - 1 + offs),
                start_time,
            );
            let end_date = NaiveDateTime::new(
                sem_start + Days::new(f_instance.day_of_week - 1 + offs),
                end_time,
            );

            event.starts((start_date, Bucharest));
            event.ends((end_date, Bucharest));

            match f_instance.periodicity {
                Periodicity::Weekly => {
                    event.recurrence(RRule::new(icalendar::Frequency::Weekly).until(sem_end))
                }
                Periodicity::OddWeekly | Periodicity::EvenWeekly => event.recurrence(
                    RRule::new(icalendar::Frequency::Weekly)
                        .interval(2)
                        .until(sem_end),
                ),
            }
            .expect("Couldn't add recurrence");

            for exdate in &exdates {
                if exdate.weekday() != start_date.weekday() {
                    continue;
                }

                event.add_multi_property(
                    "EXDATE;TZID=Europe/Bucharest",
                    &exdate
                        .and_time(start_time)
                        .format("%Y%m%dT%H%M%S")
                        .to_string(),
                );
            }

            calendar.push(event);
        }
    }

    calendar.timezone(Bucharest);

    let ics = calendar.name(&config.semester.name).to_string();

    println!("{}", ics);

    let _ = fs::write("cal.ics", ics);
}
