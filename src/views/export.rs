use std::sync::{Arc, Mutex};

use floem::event::EventPropagation;
use floem::ext_event::create_signal_from_tokio_channel;
use floem::peniko::Color;
use floem::reactive::{create_effect, create_rw_signal, RwSignal, SignalGet, SignalUpdate};
use floem::views::Decorators;
use floem::{
    event::EventListener,
    reactive::create_signal,
    style::Style,
    views::{button, h_stack, label, v_stack},
    View,
};
use stunts_engine::animations::Sequence;
use stunts_engine::editor::{Viewport, WindowSize};
use stunts_engine::export::exporter::{ExportProgress, Exporter};
use stunts_engine::timelines::SavedTimelineStateConfig;
use tokio::sync::mpsc;

use crate::editor_state::EditorState;

use super::sequence_timeline::TimelineState;

use std::thread;

// Messages we'll send to the export thread
pub enum ExportCommand {
    StartExport {
        output_path: String,
        window_size: WindowSize,
        sequences: Vec<Sequence>,
        saved_timeline_state_config: SavedTimelineStateConfig,
        video_width: u32,
        video_height: u32,
        total_duration_s: f64,
        progress_tx: mpsc::UnboundedSender<ExportProgress>,
    },
    Stop,
}

// Create a dedicated thread for handling the non-Send encoder
pub fn spawn_export_thread() -> mpsc::Sender<ExportCommand> {
    let (cmd_tx, mut cmd_rx) = mpsc::channel(1);

    thread::spawn(move || {
        // Create event loop for the thread
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            while let Some(cmd) = cmd_rx.recv().await {
                match cmd {
                    ExportCommand::StartExport {
                        output_path,
                        window_size,
                        sequences,
                        saved_timeline_state_config,
                        video_width,
                        video_height,
                        total_duration_s,
                        progress_tx,
                    } => {
                        // Create Exporter in this thread where it will be used
                        let mut exporter = Exporter::new(&output_path);

                        match exporter
                            .run(
                                window_size,
                                sequences,
                                saved_timeline_state_config,
                                video_width,
                                video_height,
                                total_duration_s,
                                progress_tx.clone(),
                            )
                            .await
                        {
                            Ok(_) => {
                                let _ = progress_tx.send(ExportProgress::Complete);
                            }
                            Err(e) => {
                                let _ = progress_tx.send(ExportProgress::Error(e.to_string()));
                            }
                        }
                    }
                    ExportCommand::Stop => break,
                }
            }
        });
    });

    cmd_tx
}

pub fn export_widget(
    editor_state: Arc<Mutex<EditorState>>,
    viewport: Arc<Mutex<Viewport>>,
    sequence_timeline_signal: RwSignal<TimelineState>,
) -> impl View {
    let (progress_tx, progress_rx) = mpsc::unbounded_channel();
    let progress = create_signal_from_tokio_channel(progress_rx);
    let is_exporting = create_rw_signal(false);
    let progress_text = create_rw_signal(String::from("Ready to export"));

    // Create the export thread and keep its sender
    let export_thread_tx = create_rw_signal(spawn_export_thread());

    create_effect(move |_| {
        if let Some(result) = progress.get() {
            match result {
                ExportProgress::Progress(percent) => {
                    progress_text.set(format!("Exporting: {:.1}%", percent));
                }
                ExportProgress::Complete => {
                    progress_text.set("Export complete!".to_string());
                    is_exporting.set(false);
                }
                ExportProgress::Error(err) => {
                    progress_text.set(format!("Export failed: {}", err));
                    is_exporting.set(false);
                }
            }
        }
    });

    v_stack((h_stack((
        button(label(move || "Export Video"))
            .style(|s| s.background(Color::rgb8(255, 25, 25)).color(Color::WHITE))
            .on_click(move |_| {
                if is_exporting.get() {
                    return EventPropagation::Stop;
                }

                is_exporting.set(true);
                let progress_tx = progress_tx.clone();

                progress_text.set("Starting export...".to_string());

                let mut editor_state = editor_state.lock().unwrap();
                let viewport = viewport.lock().unwrap();

                let window_size = WindowSize {
                    width: viewport.width as u32,
                    height: viewport.height as u32,
                };
                let mut new_state = editor_state
                    .saved_state
                    .as_mut()
                    .expect("Couldn't get Saved State")
                    .clone();

                let sequences = new_state.sequences.clone();

                let timeline_state = sequence_timeline_signal.get();
                let saved_timeline_state_config = timeline_state.to_config();
                let total_duration_s: f64 = saved_timeline_state_config
                    .timeline_sequences
                    .iter()
                    .map(|s| s.duration_ms as f64 / 1000.0)
                    .sum();

                // Send command to export thread
                let _ = export_thread_tx.get().send(ExportCommand::StartExport {
                    output_path: "D:/projects/common/stunts/output.mp4".to_string(),
                    window_size,
                    sequences: sequences.clone(),
                    saved_timeline_state_config: saved_timeline_state_config.clone(),
                    video_width: 1920,
                    video_height: 1080,
                    total_duration_s,
                    progress_tx,
                });

                EventPropagation::Stop
            })
            .disabled(move || is_exporting.get()),
        label(move || progress_text.get()),
    ))
    .style(|s| s.gap(10.0)),))
    .style(|s| s.padding(10.0))
}
