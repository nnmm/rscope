#[macro_use]
extern crate chan;
extern crate chan_signal;
extern crate jack;
extern crate input;
extern crate piston_window;

extern crate osc;

use piston_window::*;
use input::Input;
use jack::prelude as j;
use chan_signal::Signal;
use std::cmp;

use osc::notifications::*;
use osc::parseopts::*;




fn main() {
    let OscOpts { magnification: mag, samples_per_frame: spf } = get_options();


    // Signal gets a value when the OS sent a INT or TERM signal.
    let signal = chan_signal::notify(&[Signal::INT, Signal::TERM]);
    // When our work is complete, send a sentinel value on `sdone`.
    let (sdone, rdone) = chan::sync(0);
    let (snd, rcv) = chan::async();

    // Create client
    let (client, _status) = j::Client::new("rs-oscilloscope", j::client_options::NO_START_SERVER)
        .unwrap();

    // Register ports. They will be used in a callback that will be
    // called when new data is available.
    let in_a = client.register_port("in_1", j::AudioInSpec::default()).unwrap();
    let in_b = client.register_port("in_2", j::AudioInSpec::default()).unwrap();


    let mut out_a = client.register_port("rust_out_l", j::AudioOutSpec::default()).unwrap();
    let mut out_b = client.register_port("rust_out_r", j::AudioOutSpec::default()).unwrap();

    if let Some(l) = client.port_by_name("SuperCollider:out_1") {
        println!("{:?}", client.connect_ports(&l, &in_a));
    }
    if let Some(r) = client.port_by_name("SuperCollider:out_2") {
        client.connect_ports(&r, &in_b);
    }
    let process_callback = move |_: &j::Client, ps: &j::ProcessScope| -> j::JackControl {
        let mut out_a_p = j::AudioOutPort::new(&mut out_a, ps);
        let mut out_b_p = j::AudioOutPort::new(&mut out_b, ps);
        let in_a_p = j::AudioInPort::new(&in_a, ps);
        let in_b_p = j::AudioInPort::new(&in_b, ps);
        for (&ls, &rs) in in_a_p.iter().zip(in_b_p.iter()) {
            snd.send((ls, rs));
        }
        out_a_p.clone_from_slice(&in_a_p);
        out_b_p.clone_from_slice(&in_b_p);
        j::JackControl::Continue
    };


    let process = j::ClosureProcessHandler::new(process_callback);

    // Activate the client, which starts the processing.
    let active_client = j::AsyncClient::new(client, Notifications, process).unwrap();

    ::std::thread::spawn(move || run_graphics(mag, rcv, sdone));

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

fn run_graphics(mag: f64, rcv: chan::Receiver<(f32, f32)>, _: chan::Sender<()>) {
    let mut audio = rcv.iter();
    let mut window: PistonWindow =
        WindowSettings::new("rs-oscilloscope", [640, 480])
        .exit_on_esc(true).build().unwrap();
    let mut last_x = 0.0;
    let mut last_y = 0.0;
    while let Some(e) = window.next() {
        if let Input::Render(rargs) = e {
            let cx = rargs.width as f64/2.0;
            let cy = rargs.height as f64/2.0;
            let s = cmp::min(rargs.width, rargs.height) as f64/2.0;
            window.draw_2d(&e, |c, g| {
                clear([0.1, 0.1, 0.1, 1.0], g);
                for (l, r) in audio.by_ref().take(2048) {
                    let x = mag * s * l as f64 + cx;
                    let y = mag * s * r as f64 + cy;
                    let d = (x - last_x).powi(2) + (y - last_y).powi(2);
                    line([1.0, 0.8, 0.0, 1.0], // red
                         0.5,
                         [last_x, last_y, x, y],
                         c.transform, g);
                    last_x = x;
                    last_y = y;
                }
            });
        }
    }
    // Quit normally.
    // Note that we don't need to send any values. We just let the
    // sending channel drop, which closes the channel, which causes
    // the receiver to synchronize immediately and always.
}
