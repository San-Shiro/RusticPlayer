use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;
use ui_kit::organisms::debug_overlay::DebugLogItem;

pub fn format_panic_report(thread_name: &str, location: &str, payload: &str, backtrace: &std::backtrace::Backtrace) -> String {
    let now_str = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(d) => format!("{}.{:03}s epoch", d.as_secs(), d.subsec_millis()),
        Err(_) => "unknown time".into(),
    };

    format!(
        "================================================================================\n\
         RUSTICPLAYER CRASH / PANIC REPORT\n\
         Timestamp: {}\n\
         Thread: {}\n\
         Location: {}\n\
         Payload: {}\n\
         --------------------------------------------------------------------------------\n\
         Stack Backtrace:\n\
         {}\n\
         ================================================================================\n",
        now_str, thread_name, location, payload, backtrace
    )
}

pub fn init_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let thread = std::thread::current();
        let thread_name = thread.name().unwrap_or("<unnamed>");
        let location = panic_info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown location".into());

        let payload = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            (*s).to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Box<dyn Any> (non-string payload)".to_string()
        };

        let backtrace = std::backtrace::Backtrace::force_capture();
        let crash_content = format_panic_report(thread_name, &location, &payload, &backtrace);

        eprintln!("\n{}", crash_content);
        let _ = std::fs::write("rustic_panic.log", &crash_content);

        default_hook(panic_info);
    }));
}

#[derive(Clone, Debug)]
pub struct StoredLogEntry {
    pub timestamp: String,
    pub level: tracing::Level,
    pub target: String,
    pub message: String,
}

pub struct LogStore {
    entries: RwLock<VecDeque<StoredLogEntry>>,
    max_capacity: usize,
}

impl LogStore {
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: RwLock::new(VecDeque::with_capacity(capacity)),
            max_capacity: capacity,
        }
    }

    pub fn push(&self, level: tracing::Level, target: &str, message: String) {
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default();
        let total_secs = now.as_secs();
        let hours = (total_secs / 3600) % 24;
        let mins = (total_secs / 60) % 60;
        let secs = total_secs % 60;
        let ms = now.subsec_millis();
        let timestamp = format!("{:02}:{:02}:{:02}.{:03}", hours, mins, secs, ms);

        if let Ok(mut entries) = self.entries.write() {
            if entries.len() >= self.max_capacity {
                entries.pop_front();
            }
            entries.push_back(StoredLogEntry {
                timestamp,
                level,
                target: target.to_string(),
                message,
            });
        }
    }

    pub fn clear(&self) {
        if let Ok(mut entries) = self.entries.write() {
            entries.clear();
        }
    }

    pub fn get_items(&self, errors_only: bool) -> Vec<DebugLogItem> {
        if let Ok(entries) = self.entries.read() {
            entries
                .iter()
                .filter(|e| {
                    if errors_only {
                        e.level == tracing::Level::ERROR || e.level == tracing::Level::WARN
                    } else {
                        true
                    }
                })
                .map(|e| DebugLogItem {
                    timestamp: e.timestamp.clone(),
                    level: e.level.to_string(),
                    target: e.target.clone(),
                    message: e.message.clone(),
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn generate_report(
        &self,
        audio_diag: Option<&audio::AudioDiagnostics>,
        track_title: &str,
        track_artist: &str,
        pos_secs: f64,
        dur_secs: f64,
        state_str: &str,
    ) -> String {
        let mut report = String::new();
        report.push_str("=== RUSTICPLAYER DIAGNOSTIC REPORT ===\n\n");
        report.push_str(&format!("Playback State: {}\n", state_str));
        report.push_str(&format!("Active Track: {} by {}\n", track_title, track_artist));
        report.push_str(&format!("Position: {:.3}s / {:.3}s\n\n", pos_secs, dur_secs));

        if let Some(diag) = audio_diag {
            report.push_str("--- AUDIO HARDWARE & DAC ---\n");
            report.push_str(&format!("Endpoint Name: {}\n", diag.device_name));
            report.push_str(&format!("Sample Rate: {} Hz\n", diag.sample_rate));
            report.push_str(&format!("Channels: {}\n", diag.channels));
            report.push_str(&format!("Format: {}\n", diag.sample_format));
            report.push_str(&format!("Is Running: {}\n", diag.is_running));
            report.push_str(&format!("Has Error: {}\n", diag.has_error));
            report.push_str(&format!("Last Error: {:?}\n", diag.last_error));
            report.push_str(&format!("Buffer Underruns: {}\n", diag.underruns));
            report.push_str(&format!("Hardware DAC Frames: {}\n\n", diag.dac_frames));
        }

        report.push_str("--- RECENT TRACING LOGS ---\n");
        let items = self.get_items(false);
        for item in items.iter().rev().take(50) {
            report.push_str(&format!("{} [{}] {}: {}\n", item.timestamp, item.level, item.target, item.message));
        }
        report.push_str("======================================\n");
        report
    }
}

pub struct MemoryLogLayer {
    store: Arc<LogStore>,
}

impl MemoryLogLayer {
    pub fn new(store: Arc<LogStore>) -> Self {
        Self { store }
    }
}

struct FieldVisitor {
    message: String,
}

impl tracing::field::Visit for FieldVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
        } else {
            if !self.message.is_empty() {
                self.message.push(' ');
            }
            self.message.push_str(&format!("{}={:?}", field.name(), value));
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_string();
        } else {
            if !self.message.is_empty() {
                self.message.push(' ');
            }
            self.message.push_str(&format!("{}={}", field.name(), value));
        }
    }
}

impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for MemoryLogLayer {
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        let meta = event.metadata();
        let mut visitor = FieldVisitor {
            message: String::new(),
        };
        event.record(&mut visitor);

        self.store.push(
            *meta.level(),
            meta.target(),
            visitor.message,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_store_push_and_filter() {
        let store = LogStore::new(10);
        store.push(tracing::Level::INFO, "test::info", "Info message".into());
        store.push(tracing::Level::WARN, "test::warn", "Warning message".into());
        store.push(tracing::Level::ERROR, "test::error", "Error message".into());
        store.push(tracing::Level::DEBUG, "test::debug", "Debug message".into());

        let all_items = store.get_items(false);
        assert_eq!(all_items.len(), 4);

        let err_items = store.get_items(true);
        assert_eq!(err_items.len(), 2);
        assert_eq!(err_items[0].level, "WARN");
        assert_eq!(err_items[1].level, "ERROR");

        store.clear();
        assert_eq!(store.get_items(false).len(), 0);
    }

    #[test]
    fn test_generate_report() {
        let store = LogStore::new(10);
        store.push(tracing::Level::INFO, "engine", "Playback started".into());
        store.push(tracing::Level::WARN, "audio", "Buffer underrun detected".into());

        let diag = audio::AudioDiagnostics {
            device_name: "Test Headphones".into(),
            sample_rate: 48000,
            channels: 2,
            sample_format: "F32".into(),
            is_running: true,
            has_error: false,
            last_error: None,
            underruns: 5,
            dac_frames: 96000,
            dac_time_ns: 2_000_000_000,
        };

        let report = store.generate_report(
            Some(&diag),
            "Neon Horizon",
            "Vektor Prime",
            12.5,
            180.0,
            "Playing",
        );

        assert!(report.contains("=== RUSTICPLAYER DIAGNOSTIC REPORT ==="));
        assert!(report.contains("Active Track: Neon Horizon by Vektor Prime"));
        assert!(report.contains("Endpoint Name: Test Headphones"));
        assert!(report.contains("Sample Rate: 48000 Hz"));
        assert!(report.contains("Buffer Underruns: 5"));
        assert!(report.contains("Buffer underrun detected"));
    }

    #[test]
    fn test_format_panic_report() {
        let bt = std::backtrace::Backtrace::disabled();
        let report = format_panic_report(
            "main",
            "crates/app/src/main.rs:42:1",
            "explicit panic for testing crash trap",
            &bt,
        );

        assert!(report.contains("RUSTICPLAYER CRASH / PANIC REPORT"));
        assert!(report.contains("Thread: main"));
        assert!(report.contains("Location: crates/app/src/main.rs:42:1"));
        assert!(report.contains("Payload: explicit panic for testing crash trap"));
        assert!(report.contains("Stack Backtrace:"));
    }
}
