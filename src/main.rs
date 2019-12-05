//#![deny(warnings)]
extern crate clap;
use clap::{App, Arg};
use serde_derive::Deserialize;
use toml;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
extern crate hex;
extern crate midir;
extern crate rand;
use std::thread::sleep;
use std::thread;
use std::time::Duration;
use std::io::{stdin, stdout, Write};
use midir::{MidiOutput, MidiOutputPort};
use rand::Rng;
use std::i64;
#[derive(Debug, Deserialize, Clone)]
struct Config {
    synth: Option<String>,
    patch: Option<String>,
    channel: Option<u8>,
    infiniteloop: Option<bool>,
    repeat: Option<u64>,
    msgdelay: Option<u64>,
    cc: Option<Vec<CCValues>>,
    note: Option<Vec<NoteValues>>,
    sysex: Option<Vec<SysexValues>>,
    sysex_manufacturer_id: Option<u8>,
    sysex_device_id: Option<u8>,
}
impl Config {

}
#[derive(Debug, Deserialize, Clone)]
struct CCValues {
    name: Option<String>,
    number: Option<u64>,
    minnumber: Option<u64>,
    maxnumber: Option<u64>,
    value: Option<u64>,
    minvalue: Option<u64>,
    maxvalue: Option<u64>,
    pause: Option<u64>,
    maxpause: Option<u64>,
    minpause: Option<u64>,
}
impl CCValues {
    fn empty() -> Vec<CCValues> {
        let x = CCValues {
            name: Some(String::from("empty")),
            number: Some(0),
            minnumber: Some(0),
            maxnumber: Some(0),
            value: Some(0),
            minvalue: Some(0),
            maxvalue: Some(0),
            pause: Some(0),
            minpause: Some(0),
            maxpause: Some(0),
        };
        vec![x]
    }
}
#[derive(Debug, Deserialize, Clone)]
struct NoteValues {
    number: Option<u64>,
    minnumber: Option<u64>,
    maxnumber: Option<u64>,
    velocity: Option<u64>,
    minvelocity: Option<u64>,
    maxvelocity: Option<u64>,
    duration: Option<u64>,
    minduration: Option<u64>,
    maxduration: Option<u64>,
    pause: Option<u64>,
    minpause: Option<u64>,
    maxpause: Option<u64>,
}
impl NoteValues {
    fn empty() -> Vec<NoteValues> {
        let x = NoteValues {
            number: Some(129),
            minnumber: Some(0),
            maxnumber: Some(0),
            velocity: Some(0),
            minvelocity: Some(0),
            maxvelocity: Some(0),
            duration: Some(0),
            minduration: Some(0),
            maxduration: Some(0),
            pause: Some(0),
            minpause: Some(0),
            maxpause: Some(0),
        };
        vec![x]
    }
}

#[derive(Debug, Deserialize, Clone)]
struct SysexValues {
    name: Option<String>,
    message: Option<Vec<String>>,
    pause: Option<u64>,
    minpause: Option<u64>,
    maxpause: Option<u64>,
    r1minvalue: Option<u64>,
    r1maxvalue: Option<u64>,
    r2minvalue: Option<u64>,
    r2maxvalue: Option<u64>,
    r3minvalue: Option<u64>,
    r3maxvalue: Option<u64>,

}
impl SysexValues {
    fn empty() -> Vec<SysexValues> {
        let x = SysexValues {
            name: Some(String::from("empty")),
            message: Some(vec![String::from("empty")]),
            pause: Some(0),
            minpause: Some(0),
            maxpause: Some(0),
            r1minvalue: Some(0),
            r1maxvalue: Some(0),
            r2minvalue: Some(0),
            r2maxvalue: Some(0),
            r3minvalue: Some(0),
            r3maxvalue: Some(0),

        };
        vec![x]
    }
}

fn main() {
    let matches = App::new("Midi Randomer")
        .version("0.1")
        .author("Mariano Arellano <im@mariano.pw>")
        .about("A tool to send random midi and Sysex though MIDI")
        .arg(
            Arg::with_name("start")
                .short("s")
                .long("start")
                .value_name("FILE")
                .help("File to open")
                .required(false)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .value_name("port")
                .help("Port number of your MIDI interface")
                .required(false)
                .takes_value(true),
        )
        .get_matches();

    if let Some(file) = matches.value_of("start") {
    let path = Path::new(file);
    println!("Opening file {}", path.display());
        let port = if let Some(port) = matches.value_of("port") {
            port
        } else {
            let x = "ask";
            x
        };
        //let iter = iter.parse::<u64>().unwrap();
        match run(path, port) {
        Ok(_) => (),
        Err(err) => println!("Error: {}", err.description())

        }
    } else {
    println!("No argument detected. Try: ./mrandomer -s file.toml .. or run ./mrandomer --help to get all available commands");
    }


}

fn run(path: &Path, port: &str) -> Result<(), Box<Error>> {
    //Midi config
    let midi_out = MidiOutput::new("My Test Output")?;

    let out_ports = midi_out.ports();
    // Get an output port (read from console if multiple are available)
    let out_port: &MidiOutputPort = match out_ports.len() {
        0 => return Err("no output port found".into()),
        1 => {
            println!("Choosing the only available output port: {}", midi_out.port_name(&out_ports[0]).unwrap());
            &out_ports[0]
        },
        _ => {
            println!("\nAvailable output ports:");
            for (i, p) in out_ports.iter().enumerate() {
                println!("{}: {}", i, midi_out.port_name(p).unwrap());
            }
            if port != "ask"{
            let port = port.trim().parse::<usize>().unwrap();
            println!("Choosing specified output port: {}", midi_out.port_name(&out_ports[port]).unwrap());
            &out_ports[port]

            } else {
            print!("Please select output port: ");
            stdout().flush()?;
            let mut input = String::new();
            stdin().read_line(&mut input)?;
            out_ports.get(input.trim().parse::<usize>()?)
                     .ok_or("invalid output port selected")?
            }
        }
        };

    println!("\nOpening connection");
    let mut conn_out = midi_out.connect(out_port, "midir-test")?;
    println!("Connection open. Listen!");
    {
/////////////////////// End midi settings ////////
    let mut iter = 1;
    let mut rep_assigned = false;
    while iter != 0 {
    //Reading File
    // Open the path in read-only mode, returns `io::Result<File>`
    let display = path.display();
    let mut file = match File::open(&path) {
        // The `description` method of `io::Error` returns a string that
        // describes the error
        Err(why) => panic!("couldn't open {}: {}", display, why.description()),
        Ok(file) => file,
    };
     // Read the file contents into a string, returns `io::Result<usize>`
    let mut s = String::new();
    let toml_str = match file.read_to_string(&mut s) {
        Err(why) => panic!("couldn't read {}: {}", display, why.description()),
        Ok(_) => s,
    };
    let d: Config = toml::from_str(&toml_str).unwrap();

    let mut rng = rand::thread_rng();
    if rep_assigned == false {
    //iterations
    let repetitions = match d.repeat {
        Some(x) => x,
        None => 0
    };
    println!("repetitions in file {}", repetitions);
        iter += repetitions-1;
        rep_assigned = true;
        println!("{}", iter);
        println!("{}", rep_assigned);
    };

    //delay
    let msgdelay = match d.msgdelay {
        Some(x) => x,
        None => 100
    };
    let infiniteloop = match d.infiniteloop {
        Some(x) => x,
        None => false
    };
    println!("Delay between messages: {}", msgdelay);
    //delay
    let channel = match d.channel {
        Some(x) => x,
        None => 1
    };


        // Define a new scope in which the closure `play_note` borrows conn_out, so it can be called easily
        let mut send = |msg_type: u8, number: u8, value: u8, duration: u64, pause: u64| {
            const NOTE_OFF_MSG: u8 = 0x80;
            let _ = conn_out.send(&[msg_type, number, value]);
            sleep(Duration::from_millis(duration));
            let _ = conn_out.send(&[NOTE_OFF_MSG, number, value]);
            sleep(Duration::from_millis(pause));
            sleep(Duration::from_millis(msgdelay));
        };

    let ccvalues = match d.cc {
        Some(x) => x,
        None => CCValues::empty()
    };
    if ccvalues[0].name != Some(String::from("empty")){
    for (c, cc) in ccvalues.iter().enumerate(){
     let name = match &cc.name {
         Some(x) => x,
         None => "unknown",
     };
    let number = match &cc.number {
         Some(x) => x,
         None => &129,
     };
    let minnumber = match &cc.minnumber {
         Some(x) => x,
         None => number,
     };
    let maxnumber = match &cc.maxnumber {
         Some(x) => x,
         None => number,
     };
    let value = match &cc.value {
         Some(x) => x,
         None => &129,
     };
    let minvalue = match &cc.minvalue {
         Some(x) => x,
         None => &0,
     };
    let maxvalue = match &cc.maxvalue {
         Some(x) => x,
         None => &0,
     };
    let pause = match &cc.pause {
         Some(x) => x,
         None => &0,
     };
    let minpause = match &cc.minpause {
         Some(x) => x,
         None => &0,
     };
    let maxpause = match &cc.maxpause {
         Some(x) => x,
         None => &0,
     };

    let cc_ch = 0xB0 + (channel-1);


    let ccnumber: u8 = if number == &129 {
        rng.gen_range(minnumber, maxnumber) as u8
    } else {
        (*number as u8)
    };
    let ccvalue: u8 = if value == &129 {
        if minvalue == &0 && maxvalue == &0 {
            continue
        } else if  minvalue > maxvalue {
            println!("Min value of CC {} is bigger than it's max value. Can't randomize. Please fix that", ccnumber);
            continue
        } else if minvalue == maxvalue {
            println!("Min value of CC {} is equal than it's max value. Can't randomize. Please fix that", ccnumber);
            continue
        } else {
        let x = rng.gen_range(minvalue, maxvalue) as u8;
        x
        }
    } else {
        (*value as u8)
    };
    let duration = 0;
    println!("Sending {}, CC: {}, Value: {} (Minval: {}, Maxval: {})", name, ccnumber, ccvalue, minvalue, maxvalue);
    send(cc_ch, ccnumber, ccvalue , duration, *pause);
    }
};

    let notevalues = match d.note {
        Some(x) => x,
        None => NoteValues::empty()
    };
    if notevalues[0].number != Some(129){
    for (c, note) in notevalues.iter().enumerate(){
     let number = match &note.number {
         Some(x) => x,
         None => &129,
     };
    let minnumber = match &note.minnumber {
         Some(x) => x,
         None => number,
     };
    let maxnumber = match &note.maxnumber {
         Some(x) => x,
         None => number,
     };
    let velocity = match &note.velocity {
         Some(x) => x,
         None => &129,
     };
    let minvelocity = match &note.minvelocity {
         Some(x) => x,
         None => &0,
     };
    let maxvelocity = match &note.maxvelocity {
         Some(x) => x,
         None => &127,
     };
    let pause = match &note.pause {
         Some(x) => x,
         None => &0,
     };
    let minpause = match &note.minpause {
         Some(x) => x,
         None => &0,
     };
    let maxpause = match &note.maxpause {
         Some(x) => x,
         None => &0,
     };
    let duration = match &note.duration {
         Some(x) => x,
         None => &0,
     };
    let minduration = match &note.minduration {
         Some(x) => x,
         None => &0,
     };
    let maxduration = match &note.maxduration {
         Some(x) => x,
         None => &0,
     };
    let notepause: u64 = if pause == &0 {
        if maxpause != &0 {
        let x = rng.gen_range(minpause, maxpause) as u64;
        x
        } else {
        0
        }
    } else {
        (*pause as u64)
    };
    let noteduration: u64 = if duration == &0 {
        if maxduration != &0 {
        let x = rng.gen_range(minduration, maxduration) as u64;
        x
        } else {
        0
        }
    } else {
        (*duration as u64)
    };
    let notevelocity: u8 = if velocity == &129 {
        let x = rng.gen_range(minvelocity, maxvelocity) as u8;
        x
    } else {
        (*velocity as u8)
    };
    let notenumber: u8 = if number == &129 {
        if minnumber == &0 && maxnumber == &0 {
            println!("Min number and max number can't be 0");
        } else if maxnumber < minnumber {
            println!("Max number can't be smaller than Min number");
        }
        rng.gen_range(minnumber, maxnumber) as u8
    } else {
        (*number as u8)
    };
    let note_ch = 0x90 + (channel-1);

    println!("Sending note number: {}, velocity: {} (Minval: {}, Maxval: {}, Duration: {}, Pause: {})", notenumber, notevelocity, minvelocity, maxvelocity, duration, pause);
    send(note_ch, notenumber, notevelocity , noteduration, notepause );
    };
    };


//sysex!!
    println!("Sending sysex");
    let sysexvalues = match d.sysex {
        Some(x) => x,
        None => SysexValues::empty()
    };
    if sysexvalues[0].message != Some(vec!["empty".to_string()]){
    for (c, sysex) in sysexvalues.iter().enumerate(){
    let mut sysex_msg = Vec::new();
    sysex_msg.push(0xF0u8);

    //println!("Found {} sysex messages, sending {}", sysexvalues.len(), c);

    let r1minvalue = match &sysex.r1minvalue {
         Some(x) => x,
         None => &0,
     };
    let r1maxvalue = match &sysex.r1maxvalue {
         Some(x) => x,
         None => &0,
     };
     let r2minvalue = match &sysex.r2minvalue {
         Some(x) => x,
         None => &0,
     };
     let r2maxvalue = match &sysex.r2maxvalue {
         Some(x) => x,
         None => &0,
     };
    let r3minvalue = match &sysex.r3minvalue {
         Some(x) => x,
         None => &0,
     };

    let r3maxvalue = match &sysex.r3maxvalue {
         Some(x) => x,
         None => &0,
     };
     //sysex_manufacturer_id
    let manufacturer_id = match d.sysex_manufacturer_id {
        Some(x) => x,
        None => 0
    };

     //sysex_device_id
    let device_id = match d.sysex_device_id {
        Some(x) => x,
        None => 0
    };

    let message = match &sysex.message {
         Some(x) => x,
         None => continue
     };
    let mut rng = rand::thread_rng();
    sysex_msg.push(manufacturer_id);
    sysex_msg.push(device_id);
    for (_i, item) in message.iter().enumerate(){
        let raw = if item == &"r1".to_string(){
        let x = rng.gen_range(r1minvalue, r1maxvalue);
        sysex_msg.push(x as u8);

        //let r1_hex = format!("{:x}", r1);
        //r1_hex
        } else if item == &"r2".to_string(){
        let x = rng.gen_range(r2minvalue, r2maxvalue);

        sysex_msg.push(x as u8);
        //let r2_hex = format!("{:x}", r2);
        //r2_hex
        } else if item == &"r3".to_string(){
        let x = rng.gen_range(r3minvalue, r3maxvalue);
        sysex_msg.push(x as u8);
        //let r3_hex = format!("{:x}", r3);
        //r3_hex
        } else {
        let no_prefix = item.trim_start_matches("0x");
        let pbyte = u8::from_str_radix(no_prefix, 16);
        match pbyte {
        Ok(x) => sysex_msg.push(x as u8),
        Err(e) => { println!("byte malformed. Not processing sysex message: Error: {}", e); continue}
        }
        };


        }
sysex_msg.push(0xF7u8);
println!("sysex msg: {:?}", sysex_msg);
if !sysex_msg.is_empty(){
conn_out.send(&sysex_msg)?;
sleep(Duration::from_millis(msgdelay));
}
sysex_msg.clear();

    };
    };


//for (c, sysex) in d.sysex.iter().enumerate(){
// };


//loopcheck
    if !infiniteloop {
        iter-=1;
    }
}
    println!("\nClosing connection");
    // This is optional, the connection would automatically be closed as soon as it goes out of scope
    conn_out.close();
    println!("Connection closed");
    Ok(())

    }
    }
