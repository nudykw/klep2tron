use bevy::prelude::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PerfEntry {
    pub timestamp: f64,
    pub fps: f32,
    pub cpu: f32,
    pub mem: f32,
}

#[derive(Resource, Default, Serialize, Deserialize)]
pub struct PerfHistory {
    pub entries: Vec<PerfEntry>,
}

pub fn collect_perf_system(
    time: Res<Time>,
    diagnostics: Res<DiagnosticsStore>,
    mut history: ResMut<PerfHistory>,
) {
    let current_time = time.elapsed_seconds_f64();
    if history.entries.last().map_or(true, |e| (current_time - e.timestamp) >= 1.0) {
        let fps = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS).and_then(|d| d.smoothed()).unwrap_or(0.0) as f32;
        let cpu = diagnostics.get(&bevy::diagnostic::SystemInformationDiagnosticsPlugin::CPU_USAGE).and_then(|d| d.smoothed()).unwrap_or(0.0) as f32;
        let mem = diagnostics.get(&bevy::diagnostic::SystemInformationDiagnosticsPlugin::MEM_USAGE).and_then(|d| d.smoothed()).unwrap_or(0.0) as f32;
        
        history.entries.push(PerfEntry {
            timestamp: current_time,
            fps, cpu, mem
        });
    }
}

pub fn save_perf_history(
    _history: Res<PerfHistory>,
    mut exit_events: EventReader<bevy::app::AppExit>,
) {
    for _ in exit_events.read() {
        #[cfg(not(target_arch = "wasm32"))]
        if let Ok(json) = serde_json::to_string_pretty(&*_history) {
            let _ = std::fs::write("perf_metrics.json", json);
            println!("--- Performance Report Saved to perf_metrics.json ---");
        }
    }
}
