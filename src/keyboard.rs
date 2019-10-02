
use std::thread::sleep;
use std::time::Duration;
use futures::Stream;
use std::collections::HashMap;
use futures::sync::mpsc::{unbounded, UnboundedSender};
use std::io;
use midir::{MidiOutput, MidiOutputConnection};

const NOTE_ON_MSG: u8 = 0x90;
const NOTE_OFF_MSG: u8 = 0x80;
const VELOCITY: u8 = 0x64;

// Describes a note
// 
// This note is a direct MIDI representation (440Hz concert A = 69)
// along with its relative duration in number of beats
#[derive(Clone)]
pub enum Note {
    Audible(u8, f64),
    Silence(f64)
}

// Describes a succession of notes to be played
#[derive(Clone)]
pub struct Score {
    pub bpm: u8,
    pub notes: Vec<Note>
}

pub struct Sheet {
    pub scores: Vec<Score>
}

pub struct Keyboard {
    channel: UnboundedSender<Sheet>
}

impl Keyboard {
    fn from_connection(mut keyboard: MidiOutputConnection) -> Result<Keyboard, io::Error> {
        let (tx, rx) = unbounded::<Sheet>();
        // Spawn the rx part of the loop in a separate thread
        std::thread::spawn(move || {
            let stream_iter = rx.wait();
            for sheet_r in stream_iter {
                if sheet_r.is_err() {
                    continue;
                }
                let sheet = sheet_r.unwrap();
                let first_score = sheet.scores.first().unwrap();
                let bpm = first_score.bpm;
                let note_duration:f64 = 60000.0 / (bpm as f64);
                let notes = first_score.notes.iter();
                for i in notes {
                    match i {
                        Note::Audible(note, duration) => {
                            let _ = keyboard.send(&[NOTE_ON_MSG, note.clone(), VELOCITY]);
                            sleep(Duration::from_millis((note_duration * duration) as u64));
                            let _ = keyboard.send(&[NOTE_OFF_MSG, note.clone(), VELOCITY]);
                        },
                        Note::Silence(duration) => {
                            sleep(Duration::from_millis((note_duration * duration) as u64));
                        }
                    }
                }
            }
        });
        Ok(Keyboard {
            channel: tx
        })
    }
    pub fn get_ports() -> HashMap<String, usize> {
        let midi_out = MidiOutput::new("Note player").map_err(|_| io::Error::new(io::ErrorKind::NotFound, "Could not instantiate midir")).unwrap();
        Keyboard::internal_get_ports(&midi_out)
    }
    fn internal_get_ports(midi_out: &MidiOutput) -> HashMap<String, usize> {
        let mut output = HashMap::new();
        let total_ports = midi_out.port_count();
        for i in 0..total_ports {
            match midi_out.port_name(i) {
                Ok(s) => { output.insert(s, i); },
                Err(_) => ()
            }
        }
        output
    }
    pub fn play(&self, sheet: Sheet) -> Result<(), io::Error> {
        self.channel.unbounded_send(sheet).map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "MIDI output is no longer available"))
    }
    pub fn new(port: &str) -> Result<Keyboard, io::Error> {

        let midi_out = MidiOutput::new("Note player").map_err(|_| io::Error::new(io::ErrorKind::NotFound, "Could not instantiate midir"))?;
        let ports:HashMap<String, usize> = {
            let mut output = HashMap::new();
            let total_ports = midi_out.port_count();
            for i in 0..total_ports {
                match midi_out.port_name(i) {
                    Ok(s) => { output.insert(s, i); },
                    Err(_) => ()
                }
            }
            output
        };
        ports.get(port)
            .ok_or(io::Error::new(io::ErrorKind::NotFound, "Port not found"))
            .and_then(|port_id| {
                midi_out.connect(*port_id, "Keyboard player")
                    .map_err(|_| io::Error::new(io::ErrorKind::AddrInUse, "Could not bind to MIDI output"))
            })
            .and_then(|keyboard| {
                Keyboard::from_connection(keyboard)
            })
    }
}

#[test]
fn test_midi() {
    let keyboard = Keyboard::new("Microsoft GS Wavetable Synth").expect("No keyboard");
    keyboard.play(Sheet {
        scores: vec![
            Score {
                bpm: 172,
                notes: vec![
                    Note::Audible(67, 1.5),
                    Note::Audible(67, 1.5),
                    Note::Audible(70, 1.0),
                    Note::Audible(72, 1.0),
                    Note::Audible(67, 1.5),
                    Note::Audible(67, 1.5),
                ]
            }
        ]
    });
    sleep(Duration::from_millis(5000));
}