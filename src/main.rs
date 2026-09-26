use std::fs;

use clap::Parser;
use icalendar::{Calendar, Event, EventLike};
use serde::Deserialize;


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
}
#[derive(Debug, Deserialize)]
enum Periodicity {
    Weekly, OddWeekly, EvenWeekly
}

#[derive(Debug, Deserialize)]
struct Instance {
    start_time: String,
    end_time: String,
    day_of_week: u8, // starts from 1 
    periodicity: Periodicity 
}

#[derive(Debug, Deserialize)]
enum ActivityType {
    Course, Lab, Seminar
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
    semester: Semester
}

fn main() {
    let args = Args::parse();

    let contents =
        fs::read_to_string(args.config).expect("Should have been able to read the config");

    let config = yaml_serde::from_str::<Config>(&contents).expect("Can't parse the yaml");    

    println!("Total activities: {}", config.activities.len());

    let calendar = Calendar::new()
        .name(&config.semester.name);

    for activity in config.activities {
        let mut event = Event::new();
        event.location(&activity.location);
    }
}
