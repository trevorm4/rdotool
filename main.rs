#![feature(str_split_whitespace_remainder)]

use std::collections::HashMap;
use std::env;
use std::io::{self, BufRead};
use std::process;
use std::thread;
use std::time::Duration;
use uinput::event::keyboard::{Key, Keyboard, Misc};
use uinput::Device;

#[derive(Debug, Clone)]
struct Chord {
    super_key: bool,
    altgr: bool,
    ctrl: bool,
    alt: bool,
    shift: bool,
    key: Keyboard,
}

impl Chord {
    fn new(key: Keyboard) -> Self {
        Chord {
            super_key: false,
            altgr: false,
            ctrl: false,
            alt: false,
            shift: false,
            key,
        }
    }

    fn key_down(&self, device: &mut Device) -> Result<(), Box<dyn std::error::Error>> {
        if self.super_key {
            device.press(&Key::LeftMeta)?;
        }
        if self.altgr {
            device.press(&Key::RightAlt)?;
        }
        if self.ctrl {
            device.press(&Key::LeftControl)?;
        }
        if self.alt {
            device.press(&Key::LeftAlt)?;
        }
        if self.shift {
            device.press(&Key::LeftShift)?;
        }
        device.press(&self.key)?;
        device.synchronize()?;
        Ok(())
    }

    fn key_up(&self, device: &mut Device) -> Result<(), Box<dyn std::error::Error>> {
        device.release(&self.key)?;
        if self.shift {
            device.release(&Key::LeftShift)?;
        }
        if self.alt {
            device.release(&Key::LeftAlt)?;
        }
        if self.ctrl {
            device.release(&Key::LeftControl)?;
        }
        if self.altgr {
            device.release(&Key::RightAlt)?;
        }
        if self.super_key {
            device.release(&Key::LeftMeta)?;
        }
        device.synchronize()?;
        Ok(())
    }
}

fn usage() {
    println!(
        "dotool reads actions from stdin and simulates input using uinput.

The supported actions are:
    key CHORD...
    keydown CHORD...
    keyup CHORD...
    type TEXT
    keydelay MILLISECONDS
    keyhold MILLISECONDS
    typedelay MILLISECONDS
    typehold MILLISECONDS

--list-keys    Print the possible Linux keys and exit.
--version      Print the version and exit.

See 'man dotool' for the documentation."
    );
}

fn inform(msg: &str) {
    eprintln!("dotool: {}", msg);
}

fn warn(msg: &str) {
    eprintln!("dotool: WARNING: {}", msg);
}

fn parse_chord(chord_str: &str, linux_keys: &HashMap<String, Keyboard>) -> Result<Chord, String> {
    let parts: Vec<&str> = chord_str.split('+').collect();

    if parts.is_empty() {
        return Err("empty chord".to_string());
    }

    let key_part = parts[parts.len() - 1];

    let key = linux_keys
        .get(&key_part.to_lowercase())
        .ok_or_else(|| format!("impossible key for layout: {}", key_part))?
        .clone();

    let mut chord = Chord::new(key);

    // Check if uppercase letter
    if key_part.len() == 1 && key_part.chars().next().unwrap().is_uppercase() {
        chord.shift = true;
    }

    for i in 0..parts.len() - 1 {
        match parts[i].to_lowercase().as_str() {
            "super" => chord.super_key = true,
            "altgr" => chord.altgr = true,
            "ctrl" | "control" => chord.ctrl = true,
            "alt" => chord.alt = true,
            "shift" => chord.shift = true,
            _ => return Err(format!("unknown modifier: {}", parts[i])),
        }
    }

    Ok(chord)
}

fn list_keys(keys: &HashMap<String, Keyboard>) {
    let mut items: Vec<_> = keys.iter().collect();
    items.sort_by_key(|(name, _)| *name);

    let margin = items.iter().map(|(name, _)| name.len()).max().unwrap_or(0);

    for (name, key) in items {
        println!("{:<width$} {:?}", name, key, width = margin);
    }
}

fn init_linux_keys() -> HashMap<String, Keyboard> {
    use Key::*;
    let key_mappings: Vec<(&str, Keyboard)> = vec![
        ("a", Keyboard::Key(A)),
        ("b", Keyboard::Key(B)),
        ("c", Keyboard::Key(C)),
        ("d", Keyboard::Key(D)),
        ("e", Keyboard::Key(E)),
        ("f", Keyboard::Key(F)),
        ("g", Keyboard::Key(G)),
        ("h", Keyboard::Key(H)),
        ("i", Keyboard::Key(I)),
        ("j", Keyboard::Key(J)),
        ("k", Keyboard::Key(K)),
        ("l", Keyboard::Key(L)),
        ("m", Keyboard::Key(M)),
        ("n", Keyboard::Key(N)),
        ("o", Keyboard::Key(O)),
        ("p", Keyboard::Key(P)),
        ("q", Keyboard::Key(Q)),
        ("r", Keyboard::Key(R)),
        ("s", Keyboard::Key(S)),
        ("t", Keyboard::Key(T)),
        ("u", Keyboard::Key(U)),
        ("v", Keyboard::Key(V)),
        ("w", Keyboard::Key(W)),
        ("x", Keyboard::Key(X)),
        ("y", Keyboard::Key(Y)),
        ("z", Keyboard::Key(Z)),
        ("0", Keyboard::Key(_0)),
        ("1", Keyboard::Key(_1)),
        ("2", Keyboard::Key(_2)),
        ("3", Keyboard::Key(_3)),
        ("4", Keyboard::Key(_4)),
        ("5", Keyboard::Key(_5)),
        ("6", Keyboard::Key(_6)),
        ("7", Keyboard::Key(_7)),
        ("8", Keyboard::Key(_8)),
        ("9", Keyboard::Key(_9)),
        ("f1", Keyboard::Key(F1)),
        ("f2", Keyboard::Key(F2)),
        ("f3", Keyboard::Key(F3)),
        ("f4", Keyboard::Key(F4)),
        ("f5", Keyboard::Key(F5)),
        ("f6", Keyboard::Key(F6)),
        ("f7", Keyboard::Key(F7)),
        ("f8", Keyboard::Key(F8)),
        ("f9", Keyboard::Key(F9)),
        ("f10", Keyboard::Key(F10)),
        ("f11", Keyboard::Key(F11)),
        ("f12", Keyboard::Key(F12)),
        ("f13", Keyboard::Key(F13)),
        ("f14", Keyboard::Key(F14)),
        ("f15", Keyboard::Key(F15)),
        ("f16", Keyboard::Key(F16)),
        ("f17", Keyboard::Key(F17)),
        ("f18", Keyboard::Key(F18)),
        ("f19", Keyboard::Key(F19)),
        ("f20", Keyboard::Key(F20)),
        ("f21", Keyboard::Key(F21)),
        ("f22", Keyboard::Key(F22)),
        ("f23", Keyboard::Key(F23)),
        ("f24", Keyboard::Key(F24)),
        ("space", Keyboard::Key(Space)),
        ("enter", Keyboard::Key(Enter)),
        ("return", Keyboard::Key(Enter)),
        ("tab", Keyboard::Key(Tab)),
        ("backspace", Keyboard::Key(BackSpace)),
        ("escape", Keyboard::Key(Esc)),
        ("esc", Keyboard::Key(Esc)),
        ("delete", Keyboard::Key(Delete)),
        ("insert", Keyboard::Key(Insert)),
        ("home", Keyboard::Key(Home)),
        ("end", Keyboard::Key(End)),
        ("pageup", Keyboard::Key(PageUp)),
        ("pagedown", Keyboard::Key(PageDown)),
        ("pause", Keyboard::Misc(Misc::Pause)),
        ("scrolllock", Keyboard::Key(ScrollLock)),
        ("sysrq", Keyboard::Key(SysRq)),
        ("print", Keyboard::Misc(Misc::Print)),
        ("left", Keyboard::Key(Left)),
        ("right", Keyboard::Key(Right)),
        ("up", Keyboard::Key(Up)),
        ("down", Keyboard::Key(Down)),
        ("leftshift", Keyboard::Key(LeftShift)),
        ("rightshift", Keyboard::Key(RightShift)),
        ("leftctrl", Keyboard::Key(LeftControl)),
        ("rightctrl", Keyboard::Key(RightControl)),
        ("leftalt", Keyboard::Key(LeftAlt)),
        ("rightalt", Keyboard::Key(RightAlt)),
        ("leftmeta", Keyboard::Key(LeftMeta)),
        ("rightmeta", Keyboard::Key(RightMeta)),
        ("capslock", Keyboard::Key(CapsLock)),
        ("numlock", Keyboard::Key(NumLock)),
        ("minus", Keyboard::Key(Minus)),
        ("equal", Keyboard::Key(Equal)),
        ("leftbrace", Keyboard::Key(LeftBrace)),
        ("rightbrace", Keyboard::Key(RightBrace)),
        ("semicolon", Keyboard::Key(SemiColon)),
        ("apostrophe", Keyboard::Key(Apostrophe)),
        ("grave", Keyboard::Key(Grave)),
        ("backslash", Keyboard::Key(BackSlash)),
        ("comma", Keyboard::Key(Comma)),
        ("dot", Keyboard::Key(Dot)),
        ("slash", Keyboard::Key(Slash)),
    ];

    key_mappings
        .iter()
        .map(|(name, key)| (name.to_string(), *key))
        .collect()
}

fn char_to_chord(ch: char, linux_keys: &HashMap<String, Keyboard>) -> Option<Chord> {
    let (key_str, needs_shift) = match ch {
        'a'..='z' => (ch.to_string(), false),
        'A'..='Z' => (ch.to_lowercase().to_string(), true),
        '0'..='9' => (ch.to_string(), false),
        ' ' => ("space".to_string(), false),
        '\n' => ("enter".to_string(), false),
        '\t' => ("tab".to_string(), false),
        '-' => ("minus".to_string(), false),
        '=' => ("equal".to_string(), false),
        '[' => ("leftbrace".to_string(), false),
        ']' => ("rightbrace".to_string(), false),
        ';' => ("semicolon".to_string(), false),
        '\'' => ("apostrophe".to_string(), false),
        '`' => ("grave".to_string(), false),
        '\\' => ("backslash".to_string(), false),
        ',' => ("comma".to_string(), false),
        '.' => ("dot".to_string(), false),
        '/' => ("slash".to_string(), false),
        // Shifted symbols
        '!' => ("1".to_string(), true),
        '@' => ("2".to_string(), true),
        '#' => ("3".to_string(), true),
        '$' => ("4".to_string(), true),
        '%' => ("5".to_string(), true),
        '^' => ("6".to_string(), true),
        '&' => ("7".to_string(), true),
        '*' => ("8".to_string(), true),
        '(' => ("9".to_string(), true),
        ')' => ("0".to_string(), true),
        '_' => ("minus".to_string(), true),
        '+' => ("equal".to_string(), true),
        '{' => ("leftbrace".to_string(), true),
        '}' => ("rightbrace".to_string(), true),
        ':' => ("semicolon".to_string(), true),
        '"' => ("apostrophe".to_string(), true),
        '~' => ("grave".to_string(), true),
        '|' => ("backslash".to_string(), true),
        '<' => ("comma".to_string(), true),
        '>' => ("dot".to_string(), true),
        '?' => ("slash".to_string(), true),
        _ => return None,
    };

    let key = linux_keys.get(&key_str)?;
    let mut chord = Chord::new(key.clone());
    chord.shift = needs_shift;
    Some(chord)
}

fn main() {
    if let Err(e) = run() {
        inform(&e);
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let version = env!("CARGO_PKG_VERSION");
    let args: Vec<String> = env::args().collect();

    let linux_keys = init_linux_keys();

    // Parse command line arguments
    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "-h" | "--help" => {
                usage();
                return Ok(());
            }
            "--version" => {
                println!("{}", version);
                return Ok(());
            }
            "--list-keys" => {
                list_keys(&linux_keys);
                return Ok(());
            }
            _ => {
                return Err(format!("unknown argument: {}", arg));
            }
        }
    }

    let mut keyboard = uinput::default()
        .map_err(|e| format!("Failed to initialize uinput: {}", e))?
        .name("dotool keyboard")
        .map_err(|e| format!("Failed to set device name: {}", e))?
        .event(uinput::event::Keyboard::All)
        .map_err(|e| format!("Failed to set keyboard events: {}", e))?
        .create()
        .map_err(|e| format!("Failed to create keyboard device: {}", e))?;

    let mut keydelay = Duration::from_millis(2);
    let keyhold = Duration::from_millis(8);
    let mut typedelay = Duration::from_millis(2);
    let typehold = Duration::from_millis(8);

    let stdin = io::stdin();
    let reader = stdin.lock();

    for line in reader.lines() {
        let line = line.map_err(|e| e.to_string())?;
        let text = line.trim_start();

        if text.is_empty() {
            continue;
        }

        let mut parts = text.split_whitespace();

        let op = match parts.next() {
            Some(s) => s,
            None => panic!("Invalid command"),
        };

        match op {
            "key" => {
                for field in parts {
                    match parse_chord(field, &linux_keys) {
                        Ok(chord) => {
                            if let Err(e) = chord.key_down(&mut keyboard) {
                                warn(&format!("key down error: {}", e));
                            }
                            thread::sleep(keyhold);
                            if let Err(e) = chord.key_up(&mut keyboard) {
                                warn(&format!("key up error: {}", e));
                            }
                            thread::sleep(keydelay);
                        }
                        Err(e) => warn(&e),
                    }
                }
            }
            "keydown" => {
                for field in parts {
                    match parse_chord(field, &linux_keys) {
                        Ok(chord) => {
                            if let Err(e) = chord.key_down(&mut keyboard) {
                                warn(&format!("key down error: {}", e));
                            }
                            thread::sleep(keydelay);
                        }
                        Err(e) => warn(&e),
                    }
                }
            }
            "keyup" => {
                for field in parts {
                    match parse_chord(field, &linux_keys) {
                        Ok(chord) => {
                            if let Err(e) = chord.key_up(&mut keyboard) {
                                warn(&format!("key up error: {}", e));
                            }
                            thread::sleep(keydelay);
                        }
                        Err(e) => warn(&e),
                    }
                }
            }
            "keydelay" => match parts.remainder() {
                Some(s) => match s.trim().parse::<f64>() {
                    Ok(d) => keydelay = Duration::from_millis(d as u64),
                    Err(_) => warn(&format!("invalid delay: {}", text)),
                },
                None => panic!("Delay missing"),
            },
            "type" => match parts.remainder() {
                Some(s) => {
                    for ch in s.chars() {
                        if let Some(chord) = char_to_chord(ch, &linux_keys) {
                            if let Err(e) = chord.key_down(&mut keyboard) {
                                warn(&format!("type error: {}", e));
                                continue;
                            }
                            thread::sleep(typehold);
                            if let Err(e) = chord.key_up(&mut keyboard) {
                                warn(&format!("type error: {}", e));
                            }
                            thread::sleep(typedelay);
                        } else {
                            warn(&format!("cannot type character: {}", ch));
                        }
                    }
                }
                None => panic!("Missing string to type"),
            },
            "typedelay" => match parts.remainder() {
                Some(s) => match s.trim().parse::<f64>() {
                    Ok(d) => typedelay = Duration::from_millis(d as u64),
                    Err(_) => warn(&format!("invalid delay: {}", text)),
                },
                None => panic!("Missing typedelay arguments"),
            },
            _ => panic!("Unknown operation"),
        }
    }

    Ok(())
}
