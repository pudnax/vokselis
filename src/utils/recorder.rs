use color_eyre::eyre::Result;
use crossbeam_channel::{Receiver, Sender};
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
    process::{Child, Command, Stdio},
    time::Instant,
};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

use super::ImageDimentions;

use crate::utils::{create_folder, SCREENSHOTS_FOLDER, VIDEO_FOLDER};

pub enum RecordEvent {
    Start(ImageDimentions),
    Record(Vec<u8>),
    Finish,
    Screenshot((Vec<u8>, ImageDimentions)),
}

pub struct Recorder {
    sender: Sender<RecordEvent>,
    ffmpeg_installed: bool,
    pub ffmpeg_version: String,
}

impl Recorder {
    pub fn new() -> Self {
        let mut command = Command::new("ffmpeg");
        command.arg("-version");
        let (version, installed) = match command.output() {
            Ok(output) => (
                String::from_utf8(output.stdout)
                    .unwrap()
                    .lines()
                    .next()
                    .unwrap()
                    .to_string(),
                true,
            ),
            Err(e) => (e.to_string(), false),
        };

        let (tx, rx) = crossbeam_channel::unbounded();
        std::thread::spawn(move || record_thread(rx));

        Self {
            sender: tx,
            ffmpeg_installed: installed,
            ffmpeg_version: version,
        }
    }

    pub fn send(&self, event: RecordEvent) {
        if matches!(
            event,
            RecordEvent::Finish | RecordEvent::Start(_) | RecordEvent::Record(_)
        ) && !self.ffmpeg_installed
        {
            return;
        }
        self.sender.send(event).unwrap()
    }
}

struct RecorderThread {
    process: Child,
    image_dimentions: ImageDimentions,
}

fn new_ffmpeg_command(image_dimentions: ImageDimentions, filename: &str) -> Result<RecorderThread> {
    #[rustfmt::skip]
    let args = [
        "-framerate", "60",
        "-pix_fmt", "rgba",
        "-f", "rawvideo",
        "-i", "pipe:",
        "-c:v", "libx264",
        "-crf", "15",
        "-preset", "ultrafast",
        "-tune", "animation",
        "-color_primaries", "bt709",
        "-color_trc", "bt709",
        "-colorspace", "bt709",
        "-color_range", "tv",
        "-chroma_sample_location", "center",
        "-pix_fmt", "yuv420p",
        "-movflags", "+faststart",
        "-y",
    ];

    let mut command = Command::new("ffmpeg");
    command
        .arg("-video_size")
        .arg(format!(
            "{}x{}",
            image_dimentions.unpadded_bytes_per_row / 4,
            image_dimentions.height
        ))
        .args(&args)
        .arg(filename)
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    #[cfg(windows)]
    {
        const WINAPI_UM_WINBASE_CREATE_NO_WINDOW: u32 = 0x08000000;
        // Not create terminal window
        command.creation_flags(WINAPI_UM_WINBASE_CREATE_NO_WINDOW);
    }

    let child = command.spawn()?;

    Ok(RecorderThread {
        process: child,
        image_dimentions,
    })
}

fn record_thread(rx: Receiver<RecordEvent>) {
    // puffin::profile_function!();

    let mut recorder = None;

    while let Ok(event) = rx.recv() {
        match event {
            RecordEvent::Start(image_dimentions) => {
                // puffin::profile_scope!("Start Recording");

                create_folder(VIDEO_FOLDER).unwrap();
                let dir_path = Path::new(VIDEO_FOLDER);
                let filename = dir_path.join(format!(
                    "record-{}.mp4",
                    chrono::Local::now().format("%d-%m-%Y-%H-%M-%S")
                ));
                recorder =
                    Some(new_ffmpeg_command(image_dimentions, filename.to_str().unwrap()).unwrap());
            }
            RecordEvent::Record(frame) => {
                // puffin::profile_scope!("Process Frame");

                if let Some(ref mut recorder) = recorder {
                    let writer = recorder.process.stdin.as_mut().unwrap();
                    let mut writer = BufWriter::new(writer);

                    let padded_bytes = recorder.image_dimentions.padded_bytes_per_row as _;
                    let unpadded_bytes = recorder.image_dimentions.unpadded_bytes_per_row as _;
                    for chunk in frame
                        .chunks(padded_bytes)
                        .map(|chunk| &chunk[..unpadded_bytes])
                    {
                        writer.write_all(chunk).unwrap();
                    }
                    // writer.write_all(&frame).unwrap();
                    writer.flush().unwrap();
                }
            }
            RecordEvent::Finish => {
                // puffin::profile_scope!("Stop Recording");

                if let Some(ref mut process) = recorder {
                    process.process.wait().unwrap();
                }
                drop(recorder);
                recorder = None;
                eprintln!("Recording finished");
            }
            RecordEvent::Screenshot((frame, image_dimentions)) => {
                match save_screenshot(frame, image_dimentions) {
                    Ok(_) => {}
                    Err(err) => {
                        eprintln!("{err}")
                    }
                }
            }
        }
    }
}

pub fn save_screenshot(frame: Vec<u8>, image_dimentions: ImageDimentions) -> Result<()> {
    let now = Instant::now();
    let screenshots_folder = Path::new(SCREENSHOTS_FOLDER);
    create_folder(screenshots_folder)?;
    let path = screenshots_folder.join(format!(
        "screenshot-{}.png",
        chrono::Local::now().format("%d-%m-%Y-%H-%M-%S")
    ));
    let file = File::create(path)?;
    let w = BufWriter::new(file);
    let mut encoder =
        png::Encoder::new(w, image_dimentions.width as _, image_dimentions.height as _);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let padded_bytes = image_dimentions.padded_bytes_per_row as _;
    let unpadded_bytes = image_dimentions.unpadded_bytes_per_row as _;
    let mut writer = encoder
        .write_header()?
        .into_stream_writer_with_size(unpadded_bytes)?;
    for chunk in frame
        .chunks(padded_bytes)
        .map(|chunk| &chunk[..unpadded_bytes])
    {
        writer.write_all(chunk)?;
    }
    writer.finish()?;
    eprintln!("Encode image: {:#.2?}", now.elapsed());
    Ok(())
}
