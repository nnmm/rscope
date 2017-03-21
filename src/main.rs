#[macro_use]
extern crate chan;
extern crate chan_signal;
extern crate jack;
extern crate piston_window;

use piston_window::*;
use jack::prelude as j;
use chan_signal::Signal;
use std::cmp::Ordering::Equal;

struct Notifications;

impl j::NotificationHandler for Notifications {
    fn thread_init(&self, _: &j::Client) {
        println!("JACK: thread init");
    }

    fn shutdown(&mut self, status: j::ClientStatus, reason: &str) {
        println!("JACK: shutdown with status {:?} because \"{}\"",
                 status,
                 reason);
    }

    fn freewheel(&mut self, _: &j::Client, is_enabled: bool) {
        println!("JACK: freewheel mode is {}",
                 if is_enabled { "on" } else { "of" });
    }

    fn buffer_size(&mut self, _: &j::Client, sz: j::JackFrames) -> j::JackControl {
        println!("JACK: buffer size changed to {}", sz);
        j::JackControl::Continue
    }

    fn sample_rate(&mut self, _: &j::Client, srate: j::JackFrames) -> j::JackControl {
        println!("JACK: sample rate changed to {}", srate);
        j::JackControl::Continue
    }

    fn client_registration(&mut self, _: &j::Client, name: &str, is_reg: bool) {
        println!("JACK: {} client with name \"{}\"",
                 if is_reg { "registered" } else { "unregistered" },
                 name);
    }

    fn port_registration(&mut self, _: &j::Client, port_id: j::JackPortId, is_reg: bool) {
        println!("JACK: {} port with id {}",
                 if is_reg { "registered" } else { "unregistered" },
                 port_id);
    }

    fn port_rename(&mut self,
                   _: &j::Client,
                   port_id: j::JackPortId,
                   old_name: &str,
                   new_name: &str)
                   -> j::JackControl {
        println!("JACK: port with id {} renamed from {} to {}",
                 port_id,
                 old_name,
                 new_name);
        j::JackControl::Continue
    }

    fn ports_connected(&mut self,
                       _: &j::Client,
                       port_id_a: j::JackPortId,
                       port_id_b: j::JackPortId,
                       are_connected: bool) {
        println!("JACK: ports with id {} and {} are {}",
                 port_id_a,
                 port_id_b,
                 if are_connected {
                     "connected"
                 } else {
                     "disconnected"
                 });
    }

    fn graph_reorder(&mut self, _: &j::Client) -> j::JackControl {
        println!("JACK: graph reordered");
        j::JackControl::Continue
    }

    fn xrun(&mut self, _: &j::Client) -> j::JackControl {
        println!("JACK: xrun occurred");
        j::JackControl::Continue
    }

    fn latency(&mut self, _: &j::Client, mode: j::LatencyType) {
        println!("JACK: {} latency has changed",
                 match mode {
                     j::LatencyType::Capture => "capture",
                     j::LatencyType::Playback => "playback",
                 });
    }
}


fn main() {
    // Signal gets a value when the OS sent a INT or TERM signal.
    let signal = chan_signal::notify(&[Signal::INT, Signal::TERM]);
    // When our work is complete, send a sentinel value on `sdone`.
    let (sdone, rdone) = chan::sync(0);
    let (snd_l, rcv_l) = chan::async();
    let (snd_r, rcv_r) = chan::async();

    // Create client
    let (client, _status) = j::Client::new("rust_jack_simple", j::client_options::NO_START_SERVER)
        .unwrap();
    println!("{:?}", client.ports(None, None, j::PortFlags::empty()));
    // Register ports. They will be used in a callback that will be
    // called when new data is available.
    let in_a = client.register_port("rust_in_l", j::AudioInSpec::default()).unwrap();
    let in_b = client.register_port("rust_in_r", j::AudioInSpec::default()).unwrap();
    let mut out_a = client.register_port("rust_out_l", j::AudioOutSpec::default()).unwrap();
    let mut out_b = client.register_port("rust_out_r", j::AudioOutSpec::default()).unwrap();
    let process_callback = move |_: &j::Client, ps: &j::ProcessScope| -> j::JackControl {
        let mut out_a_p = j::AudioOutPort::new(&mut out_a, ps);
        let mut out_b_p = j::AudioOutPort::new(&mut out_b, ps);
        let in_a_p = j::AudioInPort::new(&in_a, ps);
        let in_b_p = j::AudioInPort::new(&in_b, ps);
        println!("Callback: {}/{}", in_a_p.len(), in_b_p.len());
        for ls in in_a_p.iter() {
            snd_l.send(*ls);
        }
        for rs in in_b_p.iter() {
            snd_r.send(*rs);
        }
        out_a_p.clone_from_slice(&in_a_p);
        out_b_p.clone_from_slice(&in_b_p);
        j::JackControl::Continue
    };
    let process = j::ClosureProcessHandler::new(process_callback);

    // Activate the client, which starts the processing.
    let active_client = j::AsyncClient::new(client, Notifications, process).unwrap();

    ::std::thread::spawn(move || run_graphics(rcv_l, rcv_r, sdone));

    // Wait for a signal or for work to be done.
    chan_select! {
        signal.recv() -> signal => {
            println!("Received signal: {:?}", signal)
        },
        rdone.recv() => {
            println!("Program completed normally.");
        }
    }
    println!("Deactivating!");
    active_client.deactivate().unwrap();
}

fn run_graphics(rcv_l: chan::Receiver<f32>, rcv_r: chan::Receiver<f32>, _: chan::Sender<()>) {
    let mut audio = rcv_l.iter().zip(rcv_r.iter());
    let mut window: PistonWindow =
        WindowSettings::new("Hello Piston!", [640, 480])
        .exit_on_esc(true).build().unwrap();
    let mut last_x = 0.0;
    let mut last_y = 0.0;
    while let Some(e) = window.next() {
        window.draw_2d(&e, |c, g| {
            clear([1.0; 4], g);
            for (l, r) in audio.by_ref().take(256) {
                let x = (100.0 * l + 100.0) as f64;
                let y = (100.0 * r + 100.0) as f64;
                line([1.0, 0.0, 0.0, 1.0], // red
                     1.0,
                     [last_x, last_y, x, y],
                     c.transform, g);
                last_x = x;
                last_y = y;
            }
        });
    }
    // Quit normally.
    // Note that we don't need to send any values. We just let the
    // sending channel drop, which closes the channel, which causes
    // the receiver to synchronize immediately and always.
}
