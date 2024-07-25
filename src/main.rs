use std::{
    fmt::Display,
    fs::DirEntry,
    io::{BufRead, Cursor, Read},
    ops::Range,
};

use eframe::NativeOptions;
use egui::{CentralPanel, Color32, FontFamily, FontId, RichText, TextStyle, Ui};

fn main() {
    let profiler = std::env::var("PROFILING").is_ok();
    if profiler {
        puffin::set_scopes_on(true);
    }

    eframe::run_native(
        "Linux Explorer",
        NativeOptions::default(),
        Box::new(|_cc| Box::<App>::default()),
    )
    .unwrap();
}

struct App {
    processes: Vec<Process>,
    profiling: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            processes: parse_processes(),
            profiling: std::env::var("PROFILING").is_ok(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        puffin::profile_function!();
        puffin::GlobalProfiler::lock().new_frame();

        if self.profiling {
            puffin_egui::profiler_window(ctx);
        }

        let mut style = (*ctx.style()).clone();

        style.text_styles = [
            (TextStyle::Heading, FontId::new(25.0, FontFamily::Monospace)),
            (TextStyle::Body, FontId::new(14.0, FontFamily::Monospace)),
            (TextStyle::Button, FontId::new(14.0, FontFamily::Monospace)),
            (
                TextStyle::Monospace,
                FontId::new(14.0, FontFamily::Monospace),
            ),
        ]
        .into();

        style.visuals.panel_fill = Color32::BLACK;

        ctx.set_style(style);

        CentralPanel::default().show(ctx, |ui| {
            ui.heading(RichText::new("Processes").color(Color32::WHITE));

            egui::ScrollArea::both().auto_shrink(false).show_rows(
                ui,
                ui.text_style_height(&TextStyle::Body),
                self.processes.len(),
                |ui, row_range| {
                    let Range { start, end } = row_range;
                    for process in &self.processes[start..end] {
                        process.show(ui);
                    }
                },
            );
        });
    }
}

/// https://docs.kernel.org/filesystems/proc.html
struct Process {
    pid: u64,
    cmdline: String,

    stats: ProcessStats,
}

impl Process {
    fn show(&self, ui: &mut Ui) {
        puffin::profile_function!();

        ui.collapsing(
            RichText::new(format!("{} {}", self.pid.to_string(), self.cmdline))
                .color(Color32::WHITE),
            |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Tcomm").italics().color(Color32::WHITE));
                    ui.label(RichText::new(&self.stats.tcomm).color(Color32::LIGHT_GRAY));
                    ui.label(RichText::new("State").italics().color(Color32::WHITE));
                    ui.label(
                        RichText::new(self.stats.state.to_string()).color(Color32::LIGHT_GRAY),
                    );
                });
            },
        );
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
    state: ProcessState,
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

    c.read_until(b' ', &mut Vec::new()).unwrap();

    let mut state_byte = vec![0; 1];
    c.read_exact(&mut state_byte).unwrap();

    let state = match state_byte[0] {
        b'R' => ProcessState::Running,
        b'S' => ProcessState::Sleeping,
        b'D' => ProcessState::UninterruptibleSleeping,
        b'Z' => ProcessState::Zombie,
        b'T' => ProcessState::Stopped,
        b'I' => ProcessState::Idle,
        b => panic!("unknown state {}", b),
    };

    ProcessStats { _pid, tcomm, state }
}

enum ProcessState {
    Running,
    Sleeping,
    UninterruptibleSleeping,
    Stopped,
    Zombie,
    Idle,
}

impl Display for ProcessState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessState::Running => write!(f, "Running"),
            ProcessState::Sleeping => write!(f, "Sleeping"),
            ProcessState::UninterruptibleSleeping => write!(f, "Uninterruptable Sleep"),
            ProcessState::Stopped => write!(f, "Stopped"),
            ProcessState::Zombie => write!(f, "Zombie"),
            ProcessState::Idle => write!(f, "Idle"),
        }
    }
}
