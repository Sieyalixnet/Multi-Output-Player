// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use rodio::cpal::traits::HostTrait;
use rodio::{cpal, source::Source, Decoder, OutputStream, Sink};
use rodio::{Device, DeviceTrait, OutputStreamHandle};
use std::borrow::BorrowMut;
use std::fs::File;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use webbrowser;

use std::io;
use std::io::BufReader;

struct OutputSink {
    device_name: String,
    handle: OutputStreamHandle,
    stream: OutputStream,
    join_handle: JoinHandle<()>,
    sink: Arc<Mutex<Sink>>,
}
impl Drop for OutputSink {
    fn drop(&mut self) {
        println!("drop sink");
    }
}

struct DevicesData {
    devices: Vec<Device>,
}

impl DevicesData {
    fn new() -> Self {
        DevicesData {
            devices: Vec::new(),
        }
    }
}

struct MusicMetaData {
    path: String,
}

impl MusicMetaData {
    fn new() -> Self {
        MusicMetaData {
            path: "".to_string(),
        }
    }
}

pub struct State {
    pub output_sink_collection: Vec<OutputSink>,
    pub devices_data: DevicesData,
    pub music_metadata: MusicMetaData,
}

impl State {
    pub fn new() -> Self {
        State {
            output_sink_collection: Vec::<OutputSink>::new(),
            devices_data: DevicesData::new(),
            music_metadata: MusicMetaData::new(),
        }
    }
}
unsafe impl Send for State {}
unsafe impl Sync for State {}
pub struct RustState(pub Arc<Mutex<State>>);

#[derive(serde::Serialize, Clone, Debug)]
struct DevicesPayload {
    devices: Vec<String>,
}

#[tauri::command]
fn refresh_devices() -> String {
    let mut device_playload = DevicesPayload { devices: vec![] };
    let host = cpal::default_host();
    let devices: Vec<Device> = host.output_devices().unwrap().collect();
    for device in devices {
        let name = device.name().unwrap();
        device_playload.devices.push(name);
    }
    let res = serde_json::to_string_pretty(&device_playload);
    match res {
        Ok(s) => s,
        Err(e) => e.to_string(),
    }
}

#[derive(serde::Serialize, Clone, Debug)]
struct FilePayload {
    name: String,
    path: String,
    duration: u128,
    sample_rate: u32,
    format: String,
}

#[tauri::command]
fn detect_file(path: &str) -> String {
    let lower_path = path.to_lowercase();
    let mut file_format = "";
    if lower_path.ends_with(".flac") {
        file_format = "FLAC";
    } else if lower_path.ends_with(".mp3") {
        file_format = "MP3";
    } else if lower_path.ends_with(".wav") {
        file_format = "WAV";
    } else {
        return "unknown".to_string();
    }
    println!("file path {}", path);
    if let Ok(item) = File::open(path) {
        let file = BufReader::new(item);
        let source = Decoder::new(file).unwrap();
        println!(
            "duration {:?}",
            source.total_duration().unwrap().as_millis()
        );
        println!("sample rate {:?}", source.sample_rate());
        let p = Path::new(path);
        let filename = p.file_name().unwrap().to_str().unwrap().to_string();
        let file_payload = FilePayload {
            name: filename,
            path: path.to_string(),
            duration: source.total_duration().unwrap().as_millis(),
            sample_rate: source.sample_rate(),
            format: file_format.to_string(),
        };
        let res = serde_json::to_string_pretty(&file_payload);
        match res {
            Ok(s) => s,
            Err(e) => e.to_string(),
        }
    } else {
        return "unknown".to_string();
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
struct OutputDevicePlayload {
    name: String,
    volumn: f32,
}

#[tauri::command]
async fn play(
    path: String,
    outputDevices: Vec<OutputDevicePlayload>,
    state: tauri::State<'_, RustState>,
) -> Result<(), ()> {
    let host = cpal::default_host();
    let devices: Vec<Device> = host.output_devices().unwrap().collect();
    for dev in devices {
        let name = dev.name().unwrap();
        // if outputDevices.contains(&name) == false {
        //     continue;
        // }
        let mut skip = true;
        let mut volumn: f32 = 1.0;
        for item in outputDevices.iter() {
            if item.name == name {
                volumn = item.volumn;
                skip = false;
            }
        }
        if skip == true {
            continue;
        }
        let (_outputstream, stream_handle) = OutputStream::try_from_device(&dev).unwrap();
        let file = BufReader::new(File::open(&path).unwrap());
        let source = Decoder::new(file).unwrap();
        println!(
            "duration {:?}",
            source.total_duration().unwrap().as_millis()
        );
        println!("sample rate {:?}", source.sample_rate());
        println!("name {:?} volumn {:?}", name, volumn);
        // Play the sound directly on the device
        let sink = Arc::new(Mutex::new(rodio::Sink::try_new(&stream_handle).unwrap()));
        let sink_clone = Arc::clone(&sink);
        let playback_thread = thread::spawn(move || {
            let sink = sink_clone.lock().unwrap();
            sink.set_volume(volumn);
            sink.append(source);
        });

        let arc_state = state.0.clone();
        let mut state = arc_state.lock().unwrap();
        state.output_sink_collection.push(OutputSink {
            device_name: name,
            handle: stream_handle,
            stream: _outputstream,
            join_handle: playback_thread,
            sink: sink,
        });
    }
    Ok(())
}

#[tauri::command]
async fn stop(state: tauri::State<'_, RustState>) -> Result<(), ()> {
    let arc_state = state.0.clone();
    let mut state = arc_state.lock().unwrap();

    for i in 0..state.output_sink_collection.len() {
        let sink = state.output_sink_collection[i].sink.clone();
        sink.lock().unwrap().stop();
        // state.output_sink_collection[i].join_handle.join().unwrap();
    }
    state.output_sink_collection.clear();
    Ok(())
}

#[tauri::command]
async fn pause(state: tauri::State<'_, RustState>) -> Result<(), ()> {
    let arc_state = state.0.clone();
    let mut state = arc_state.lock().unwrap();

    for i in 0..state.output_sink_collection.len() {
        let sink = state.output_sink_collection[i].sink.clone();
        sink.lock().unwrap().pause();
        // state.output_sink_collection[i].join_handle.join().unwrap();
    }

    Ok(())
}

#[tauri::command]
async fn set_volumn(
    name: String,
    volumn: f32,
    state: tauri::State<'_, RustState>,
) -> Result<(), ()> {
    let arc_state = state.0.clone();
    let mut state = arc_state.lock().unwrap();
    for i in 0..state.output_sink_collection.len() {
        let item = &state.output_sink_collection[i];
        if item.device_name == name {
            let sink = item.sink.clone();
            sink.lock().unwrap().set_volume(volumn);
        }
    }

    Ok(())
}

#[tauri::command]
async fn resume(state: tauri::State<'_, RustState>) -> Result<(), ()> {
    let arc_state = state.0.clone();
    let mut state = arc_state.lock().unwrap();

    for i in 0..state.output_sink_collection.len() {
        let sink = state.output_sink_collection[i].sink.clone();
        sink.lock().unwrap().play();
        // state.output_sink_collection[i].join_handle.join().unwrap();
    }

    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
struct GetPositionPayload {
    duration: u128,
    is_paused: bool,
}

#[tauri::command]
fn get_pos(state: tauri::State<'_, RustState>) -> String {
    let arc_state = state.0.clone();
    let mut state = arc_state.lock().unwrap();
    if state.output_sink_collection.len() == 0 {
        return "NONE".to_string();
    }
    let sink = state.output_sink_collection[0].sink.clone();
    let s = sink.lock().unwrap();
    let dur = s.get_pos();
    let is_paused = s.is_paused();
    let res = serde_json::to_string_pretty(&GetPositionPayload {
        duration: dur.as_millis(),
        is_paused: is_paused,
    });
    match res {
        Ok(s) => s,
        Err(e) => e.to_string(),
    }
}

#[tauri::command]
async fn goto(ms: u64, state: tauri::State<'_, RustState>) -> Result<(), ()> {
    let arc_state = state.0.clone();
    let mut state = arc_state.lock().unwrap();
    println!("goto {}", ms);
    for i in 0..state.output_sink_collection.len() {
        let sink = state.output_sink_collection[i].sink.clone();
        sink.lock()
            .unwrap()
            .try_seek(Duration::from_millis(ms as u64))
            .unwrap();
        // state.output_sink_collection[i].join_handle.join().unwrap();
    }
    Ok(())
}

#[tauri::command]
async fn open(path: String) -> Result<(), ()> {
    webbrowser::open(&path).expect("can not open web");
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let shared_data = Arc::new(Mutex::new(State::new()));
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .manage(RustState(shared_data))
        .invoke_handler(tauri::generate_handler![
            open,
            refresh_devices,
            play,
            stop,
            pause,
            resume,
            get_pos,
            goto,
            detect_file,
            set_volumn
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
