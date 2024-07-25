use std::{
    fs::DirEntry,
    io::{BufRead, Cursor},
};

use eframe::NativeOptions;
use egui::{CentralPanel, Color32, FontFamily, FontId, RichText, TextStyle, Ui};

fn main() {
    eframe::run_native(
        "Linux Explorer",
        NativeOptions::default(),
        Box::new(|_cc| Ok(Box::<App>::default())),
    )
    .unwrap();
}

struct App {
    processes: Vec<Process>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            processes: parse_processes(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut style = (*ctx.style()).clone();

        style.text_styles = [
            (TextStyle::Heading, FontId::new(25.0, FontFamily::Monospace)),
            (TextStyle::Body, FontId::new(14.0, FontFamily::Monospace)),
        ]
        .into();

        style.visuals.panel_fill = Color32::BLACK;

        ctx.set_style(style);

        CentralPanel::default().show(ctx, |ui| {
            ui.heading(RichText::new("Processes").color(Color32::WHITE));

            egui::ScrollArea::both().show(ui, |ui| {
                for process in &self.processes {
                    process.show(ui);
                }
            });
        });
    }
}

struct Process {
    pid: u64,
    cmdline: String,

    stats: ProcessStats,
}

impl Process {
    fn show(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new(self.pid.to_string()).color(Color32::WHITE));
            ui.label(RichText::new(&self.cmdline).color(Color32::WHITE));
        });

        ui.horizontal(|ui| {
            ui.label(RichText::new("Tcomm:").strong().color(Color32::WHITE));
            ui.label(RichText::new(&self.stats.tcomm).color(Color32::LIGHT_GRAY));
        });

        ui.separator();
    }
}

fn parse_processes() -> Vec<Process> {
    let mut processes = Vec::new();

    for entry in std::fs::read_dir("/proc").unwrap() {
        match entry {
            Ok(entry) => {
                if let Ok(pid) = entry.file_name().into_string().unwrap().parse::<u64>() {
                    let cmdline = std::fs::read_to_string(entry.path().join("cmdline"))
                        .unwrap()
                        .replace('\0', " ");
                    let stats = parse_stats(&entry);
                    let process = Process {
                        pid,
                        cmdline,
                        stats,
                    };
                    processes.push(process);
                }
            }
            Err(err) => panic!("Err reading dir entry: {}", err),
        }
    }

    processes
}

struct ProcessStats {
    _pid: u64,
    tcomm: String,
}

fn parse_stats(entry: &DirEntry) -> ProcessStats {
    let bytes = std::fs::read(entry.path().join("stat")).unwrap();
    let mut c = Cursor::new(bytes);

    let mut pid_bytes = Vec::new();
    c.read_until(b' ', &mut pid_bytes).unwrap();
    let _pid = String::from_utf8(pid_bytes)
        .unwrap()
        .trim()
        .parse::<u64>()
        .unwrap();

    let mut tcomm_bytes = Vec::new();
    c.read_until(b')', &mut tcomm_bytes).unwrap();
    let tcomm = String::from_utf8(tcomm_bytes).unwrap();
    let tcomm = tcomm[1..tcomm.len() - 1].to_string();

    ProcessStats { _pid, tcomm }
}
