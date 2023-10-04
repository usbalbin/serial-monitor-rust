#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// hide console window on Windows in release
extern crate core;
extern crate csv;
extern crate preferences;
extern crate serde;

use std::cmp::max;

use serial_monitor_rust::{data::{DataContainer, Packet}, serial::DefaultReader};

fn split(payload: &str) -> Vec<f32> {
    let mut split_data: Vec<&str> = vec![];
    for s in payload.split(':') {
        split_data.extend(s.split(','));
    }
    split_data
        .iter()
        .map(|x| x.trim())
        .flat_map(|x| x.parse::<f32>())
        .collect()
}

fn parser(packet: Packet<String>, data: &mut DataContainer<String>, failed_format_counter: &mut i32) -> bool {
    if packet.payload.is_empty() {
        return false;
    }
     
    data.raw_traffic.push(packet.clone());
    let split_data = split(&packet.payload);
    if data.dataset.is_empty() || *failed_format_counter > 10 {
        // resetting dataset
        data.dataset = vec![vec![]; max(split_data.len(), 1)];
        if data.names.len() != split_data.len() {
            data.names = (0..max(split_data.len(), 1))
                .map(|i| format!("Column {i}"))
                .collect();
        }
        *failed_format_counter = 0;
        // println!("resetting dataset. split length = {}, length data.dataset = {}", split_data.len(), data.dataset.len());
    } else if split_data.len() == data.dataset.len() {
        // appending data
        for (i, set) in data.dataset.iter_mut().enumerate() {
            set.push(split_data[i]);
            *failed_format_counter = 0;
        }
        data.time.push(packet.relative_time);
        data.absolute_time.push(packet.absolute_time);
        if data.time.len() != data.dataset[0].len() {
            // resetting dataset
            data.time = vec![];
            data.dataset = vec![vec![]; max(split_data.len(), 1)];
            if data.names.len() != split_data.len() {
                data.names = (0..max(split_data.len(), 1))
                    .map(|i| format!("Column {i}"))
                    .collect();
            }
        }
    } else {
        // not same length
        *failed_format_counter += 1;
        // println!("not same length in main! length split_data = {}, length data.dataset = {}", split_data.len(), data.dataset.len())
    }

    true
}

fn main() {
    serial_monitor_rust::app(DefaultReader, parser);
}
