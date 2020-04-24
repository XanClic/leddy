mod keyboard;
mod types;

use keyboard::Keyboard;
use types::{Color, ColorParam, ColorMethods, Direction, Gradient};


fn print_usage() {
    eprintln!("Usage: leddy [global switches] <[effect/]{{parameters...}}>

Effect parameters are separated by slashes.


Global switches are options that control leddy’s overall behavior:
  --help: Prints this text and exits


Effects:
  · all-keys (default): Set all keys’ colors.  Effectively the same as
                        “gradient”, unless color=stdin.  Then, RGB values are
                        read from stdin (format RRGGBB in hex, separated by LF).
    Parameters: color

  · pulse: Turn all LEDs on and off in a pulsing fashion
    Parameters: color, speed

  · wave: Activate LEDs like a wave rolling over the keyboard
    Parameters: color, speed, direction

  · reactive: Activate an LED when its respective key is pressed/released
    Parameters: color, speed, keyup/keydown

  · reactive-ripple: Activate sourrounding LEDs when a key is pressed/released
                     (sending a rippling wave over the keyboard)
    Parameters: color, speed, keyup/keydown

  · rain: Like wave, but activate only a small number of random LEDs per
          row/column
    Parameters: color, speed, direction

  · gradient: Create a static gradient
    Parameters: color, speed, direction

  · fade: Fade all LEDs simultaneously through a gradient
    Parameters: color, speed

Parameters:
  · color=<color parameter>: Sets the effect color
    · rainbow: A rainbow
    · random[ized]: Random colors, often on the rainbow spectrum
    · rgb:RRGGBB: A single color by its HTML notation
    · gradient:{{RRGGBB@index,}}: A gradient (up to ten colors),
                                indices are in the [0, 100] range
                                (only works for “gradient” and “fade”)
    · stdin (only for “all-keys”): Read all keys’ colors from stdin
    (Default: rainbow)

  · speed=<0..100>: Sets an effect’s speed.  Some effects may work with speeds
                    above 100.
    (Default: 50)

  · direction=<right|left|down|up>: Sets some effects’ target direction (i.e.,
                                    “right” means from left to right, etc.)
    (Default: right)

  · keyup/keydown: These choose the trigger event for the “reactive” events.
    (Default: keydown)");
}

fn strip_prefix<'a>(string: &'a str, prefix: &str) -> Option<&'a str> {
    if string.starts_with(prefix) {
        Some(string.split_at(prefix.len()).1)
    } else {
        None
    }
}

fn count_chr(string: &str, chr: char) -> usize {
    string.chars().fold(0, |sum, c| sum + (c == chr) as usize)
}

fn main() {
    /* Skip argv[0] */
    let argv: Vec<String> = std::env::args().skip(1).collect();

    /* Look for global switches before trying to open the keyboard */
    for arg in &argv {
        if !arg.starts_with("-") {
            continue;
        }

        match arg.as_str() {
            "-h" | "-?" | "--help" => {
                print_usage();
                std::process::exit(0);
            }

            x => {
                eprintln!("Unrecognized switch “{}”", x);
                eprintln!("");
                print_usage();
                std::process::exit(1);
            }
        }
    }

    let kbd = Keyboard::new();

    for arg in &argv {
        let mut effect = None;
        let mut cp = ColorParam::Rainbow;
        let mut speed = 50;
        let mut direction = Direction::Right;
        let mut keyup = false;

        for param in arg.split('/') {
            let mut ps = param.splitn(2, '=');
            let pkey = ps.next().unwrap();

            match pkey {
                "color" => {
                    let c = ps.next().unwrap();

                    if c == "rainbow" {
                        cp = ColorParam::Rainbow;
                    } else if c == "random" || c == "randomized" {
                        cp = ColorParam::Randomized;
                    } else if let Some(rgb) = strip_prefix(c, "rgb:") {
                        cp = ColorParam::Color(Color::from_str(rgb));
                    } else if let Some(gradient) = strip_prefix(c, "gradient:") {
                        let count_m_1 = std::cmp::max(count_chr(gradient, ','), 1);

                        cp = ColorParam::Gradient(Gradient {
                            colors:
                                gradient.split(',').enumerate().map(|(index, gci)| {
                                    let mut gcis = gci.splitn(2, '@');
                                    let cols = gcis.next().unwrap();

                                    let coli =
                                        if let Some(is) = gcis.next() {
                                            is.parse().unwrap()
                                        } else {
                                            ((index * 100) / count_m_1) as u8
                                        };

                                    (Color::from_str(cols), coli)
                                }).collect()
                        });
                    } else if c == "stdin" {
                        eprintln!("Reading colors from stdin not yet supported");
                        std::process::exit(1);
                    } else {
                        eprintln!("Unrecognized color parameter “{}”", c);
                        std::process::exit(1);
                    }
                }

                "speed" => {
                    speed = ps.next().unwrap().parse().unwrap();
                }

                "direction" => {
                    direction =
                        match ps.next().unwrap() {
                            "right" => Direction::Right,
                            "left" => Direction::Left,
                            "down" => Direction::Down,
                            "up" => Direction::Up,

                            x => {
                                eprintln!("Unrecognized direction “{}”", x);
                                std::process::exit(1);
                            }
                        }
                }

                "keyup" => {
                    keyup = true;
                }

                "keydown" => {
                    keyup = false;
                }

                _ => {
                    if effect.is_none() {
                        effect = Some(pkey);
                    } else {
                        eprintln!("Unrecognized parameter key “{}”", pkey);
                        std::process::exit(1);
                    }
                }
            }
        }

        match effect.unwrap_or("all-keys") {
            "all-keys"          => kbd.gradient(&cp),
            "pulse"             => kbd.pulse(&cp, speed),
            "wave"              => kbd.wave(&cp, speed, direction),
            "reactive"          => kbd.reactive(&cp, speed, keyup),
            "reactive-ripple"   => kbd.reactive_ripple(&cp, speed, keyup),
            "rain"              => kbd.rain(&cp, speed, direction),
            "gradient"          => kbd.gradient(&cp),
            "fade"              => kbd.fade(&cp, speed),

            x => {
                eprintln!("Unrecognized effect “{}”", x);
                eprintln!("");
                print_usage();
                std::process::exit(1);
            }
        }
    }
}
