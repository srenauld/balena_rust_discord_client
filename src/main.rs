extern crate midir;
extern crate serenity;

use std::env;

pub mod keyboard;

pub use keyboard::{Sheet, Score, Note, Keyboard};
use serenity::{
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

struct Handler {
    keyboard: Keyboard
}

impl EventHandler for Handler {
    // Our bot will play a sound whenever a certain message is received
    //
    // To do this, we'll bind to `message()` and react based on it.
    fn message(&self, _ctx: Context, msg: Message) {
        if msg.content == "!alert" {
            // Our keyboard is part of our struct, all we need to do is send
            // what to play
            self.keyboard.play(Sheet {
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
            }).expect("Could not play music");
        }
    }

    // This is called when the bot is connected and a READY event has been fired
    // through the websocket through which we are receiving Discord events
    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");

    // Prepare the keyboard
    let midi_out = env::var("MIDI_CHANNEL")
        .expect(
            &format!("MIDI channel not specified. Possible choices: {}", Keyboard::get_ports().keys().map(|r| r.as_str()).collect::<Vec<&str>>().join(", "))
        );
    let keyboard = Keyboard::new(&midi_out).expect("Unknown MIDI channel");

    let mut client = Client::new(&token, Handler {
        keyboard: keyboard
    }).expect("Error creating client");

    // And we start the client, blocking this thread until it stops!
    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
}