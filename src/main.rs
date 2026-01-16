use clap::Parser;
use std::fs::File;
use rodio::{Decoder, OutputStream, source::Source};

// Derive from parser to allow direct arg processing, debug for CLI options
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
// A struct with the CLI argument data for the metronome tool
struct Metronome {
    // The beats per minute of the metronome (default: 100bpm)
    #[arg(short, long, default_value_t = 100)]
    bpm: u16,
    // Path to audio file to play as metronome (default: audio/sound.wav)
    // Sourced royalty free as Metronome SFX by P4WZ on Sample Focus
    #[arg(short, long, default_value_t = String::from("audio/sound.wav"))]
    file: String,    
}

fn main() {
    // Creating the metronome struct instance, pulling from CLI args
    let metronome = Metronome::parse();

    println!("BPM: {:?}, file: {:?}", metronome.bpm, metronome.file);

    // Get an output stream handle to the default physical sound device.
    // Note that the playback stops when the stream_handle is dropped.//!`
    let stream_handle = rodio::OutputStreamBuilder::open_default_stream()
        .expect("open default audio stream");
    let _sink = rodio::Sink::connect_new(&stream_handle.mixer());
    // Load a sound from the provided argument file, path relative to Cargo.toml
    let file = File::open(&metronome.file).unwrap();
    // Decode the sound file into a source
    let source = Decoder::try_from(file).unwrap();
    // Play the sound directly on the device
    stream_handle.mixer().add(source);

    // The sound plays in a separate audio thread,
    // so we need to keep the main thread alive while it's playing.
    std::thread::sleep(std::time::Duration::from_secs(5));

}
