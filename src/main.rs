use clap::Parser;
use std::io::Cursor;
use std::num::NonZeroU16;
use std::ops::RangeInclusive;
use rodio::{Decoder, source::{Source, LimitSettings}, cpal::BufferSize};
use tokio::time;
use std::time::Duration;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};

// A range for the volume to fall within (all valid u8 values up to 100)
const U8MAX100: RangeInclusive<usize> = 0..=100;

// A function (direct from clap documentation) to parse and limit values for volume
fn max100(input: &str) -> Result<u8, String> {
    // Parse valid input values as integers
    let value: usize = input
        .parse()
        .map_err(|_| format!("{} is not an integer", input))?;
    // A strictly positive u16 returns the Ok result, else format Err
    if U8MAX100.contains(&value) {
        Ok(value as u8)
    } else {
        Err(format!(
                "volume not a positive integer from {} to {}",
                U8MAX100.start(),
                U8MAX100.end()
                ))
    }
}

// Derive from parser to allow direct arg processing
#[derive(Parser)]
#[command(name = "tempo")]
#[command(version = "0.5.0")]
#[command(about = "A cli metronome application", long_about = None)]
// A struct with the CLI argument data for the metronome tool
struct Cli {
    // The beats per minute of the metronome (default: 100bpm)
    #[arg(short, long, default_value_t = NonZeroU16::new(100).unwrap())]
    bpm: NonZeroU16,
    // Volume from 0 to 100
    #[arg(short, long, default_value_t = 5, value_parser = max100)]
    volume: u8,
}

//#[tokio::main]
/*async */fn main()/* -> Result<()> */{
    // Creating the metronome struct instance, pulling from CLI args
    let metronome = Cli::parse();    

    // Calculate the time per beat as determined per bpm (in microseconds)
    let time_per_beat = Duration::from_micros(60_000_000/(metronome.bpm.get()) as u64);
    
    // Retrieve the device's default output stream with the smallest possible buffer
    let stream_handle = rodio::OutputStreamBuilder::from_default_device()
        .expect("output device exists")
        .with_buffer_size(BufferSize::Fixed(64))
        .open_stream()
        .expect("open default audio stream");

    // Load the default sound file, using macro include_bytes! for compile-time inclusion
    let file_bytes: &'static [u8] = include_bytes!("audio/sound.wav");
    let cursor = Cursor::new(file_bytes);
   
    // Create sound limiting settings so the noise isn't harsh
    let settings = LimitSettings::new()
        .with_threshold(-6.0)
        .with_knee_width(225.0)
        .with_attack(Duration::from_micros(50))
        .with_release(Duration::from_micros(50));
    
    // Decode the sound file into a source, limiting the volume and sound peaks
    let source = Decoder::try_from(cursor)
        .expect("input file is valid")
        .amplify(0.1 * f32::from(metronome.volume))
        .limit(settings);    

    // Create a buffered version of the source to repeatedly pull from
    let buffer = source.buffered();

    // Asynchronous runtime to repeat bpm counter
    let rt = tokio::runtime::Runtime::new().unwrap();
    // Blocking (only task)
    rt.block_on(async {
        // Create a visual bpm indicator inside the async runtime
        let indicator = ProgressBar::with_draw_target(Some(1), ProgressDrawTarget::stdout_with_hz(255));
        indicator.set_prefix(format!("{} bpm", metronome.bpm));
        indicator.set_message("Ctrl+C to exit");
        indicator.set_style(ProgressStyle::with_template("{prefix:} {bar:^.white/black} {msg:}").unwrap());
        // Designate the time interval of the metronome
        let mut interval = time::interval(time_per_beat);
        // Move the indicator from 0% to 100%
        indicator.inc(1);
        loop {
            // Clone the metronome sound that we're repeating
            let sound = buffer.clone();
            // Sleep a small period of time to allow the indicator to flicker
            time::sleep(interval.period().div_f32(20.0)).await;
            // Move the indicator back to 0%
            indicator.dec(1);
            // Now we await the time interval (previous operations should take less than 1 interval)
            interval.tick().await;
            // Add the cloned sound to the buffer to play it
            stream_handle.mixer().add(sound);
            // Now we can move the indicator back to 100% while the sound plays
            indicator.inc(1);
        }
    });
}
