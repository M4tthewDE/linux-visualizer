use eframe::NativeOptions;
use egui::{CentralPanel, Ui};

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
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Processes");

            egui::ScrollArea::vertical().show(ui, |ui| {
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
}

impl Process {
    fn show(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(self.pid.to_string());
            ui.separator();
            ui.label(&self.cmdline);
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
                    let process = Process { pid, cmdline };
                    processes.push(process);
                }
            }
            Err(err) => panic!("Err reading dir entry: {}", err),
        }
    }

    processes
}
