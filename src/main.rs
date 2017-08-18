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
    let o = get_options();


    // Signal gets a value when the OS sent a INT or TERM signal.
    let signal = chan_signal::notify(&[Signal::INT, Signal::TERM]);
    // When our work is complete, send a sentinel value on `sdone`.
    let (sdone, rdone) = chan::sync(0);
    let (snd, rcv) = chan::async();

    // Create client
    let (client, _status) = j::Client::new("rscope", j::client_options::NO_START_SERVER)
        .unwrap();

    println!("Sample rate is {}", client.sample_rate());

    // Register ports. They will be used in a callback that will be
    // called when new data is available.
    let in_a = client.register_port("in_1", j::AudioInSpec::default()).unwrap();
    let in_b = client.register_port("in_2", j::AudioInSpec::default()).unwrap();

    let process_callback = move |_: &j::Client, ps: &j::ProcessScope| -> j::JackControl {
        let in_a_p = j::AudioInPort::new(&in_a, ps);
        let in_b_p = j::AudioInPort::new(&in_b, ps);
        for (&ls, &rs) in in_a_p.iter().zip(in_b_p.iter()) {
            snd.send((ls, rs));
        }
        j::JackControl::Continue
    };


    let process = j::ClosureProcessHandler::new(process_callback);

    // Activate the client, which starts the processing.
    let active_client = j::AsyncClient::new(client, Notifications, process).unwrap();

    ::std::thread::spawn(move || run_graphics(o, rcv, sdone));

    // Wait for a signal or for work to be done.
    chan_select! {
        signal.recv() -> signal => {},
        rdone.recv() => {}
    }
    active_client.deactivate().unwrap();
}

fn run_graphics(o: OscOpts, rcv: chan::Receiver<(f32, f32)>, _: chan::Sender<()>) {
    let OscOpts { magnification: mag, samples_per_frame: spf } = o;
    let mut audio = rcv.iter();
    let mut window: PistonWindow =
        WindowSettings::new("rscope", [640, 480])
        .exit_on_esc(true).build().unwrap();

    // 255*(1-powf(d,0.077)

    /*
    ofSetColor(50, 255, 50, 30);
    shapeMesh.disableColors();
    ofSetLineWidth(20.0);
    shapeMesh.draw();

    ofSetColor(50, 255, 50, 50);
    shapeMesh.disableColors();
    ofSetLineWidth(5.0);
    shapeMesh.draw();

    ofSetColor(75, 255, 75, 50);
    shapeMesh.disableColors();
    ofSetLineWidth(2.5);
    shapeMesh.draw();

    shapeMesh.enableColors();
    ofSetLineWidth(1.0);
    shapeMesh.draw();
    */
    let col_fun = |d: f32| { (1.0-d.powf(0.077)).max(0.0).min(1.0) };

    let mut last_l = 0.0;
    let mut last_r = 0.0;
    let mut last_x = 0.0;
    let mut last_y = 0.0;

    while let Some(e) = window.next() {
        if let Input::Render(rargs) = e {
            let cx = rargs.width as f64/2.0;
            let cy = rargs.height as f64/2.0;
            let s = cmp::min(rargs.width, rargs.height) as f64/2.0;
            window.draw_2d(&e, |c, g| {
                clear([0.1, 0.1, 0.1, 0.8], g);
                for (l, r) in audio.by_ref().take(spf) {
                    let x = (l as f64).mul_add(mag * s, cx);
                    let y = (r as f64).mul_add(- mag * s, cy);
                    let d = ((l - last_l).powi(2) + (r - last_r).powi(2)).sqrt();
                    line([1.0, 0.8, 0.0, col_fun(d)],
                         0.7,
                         [last_x, last_y, x, y],
                         c.transform, g);
                    last_l = l;
                    last_r = r;
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
