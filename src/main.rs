mod keyboard;
mod software_effects;
mod types;

use keyboard::Keyboard;
use software_effects::screen_capture::screen_capture;
use types::{Color, ColorParam, ColorMethods, Direction, Gradient, KeyMap};


fn print_usage() {
    eprintln!("Usage: leddy [global switches] <[effect/]{{parameters...}}>

Effect parameters are separated by slashes.


Global switches are options that control leddy’s overall behavior:
  --help, -h
        Prints this text and exits

  --profile=<profile>, -p=<profile>
        Selects the profile to use.

        (Default: 1)


Effects:
  · all-keys (default)
        Set all keys’ colors.  Effectively the same as “gradient”, unless
        color=stdin.  Then, RGB values are read from stdin (format RRGGBB in
        hex, separated by LF).

        Parameters: color

  · pulse
        Turn all LEDs on and off in a pulsing fashion

        Parameters: color, speed

  · wave
        Activate LEDs like a wave rolling over the keyboard

        Parameters: color, speed, direction

  · reactive
        Activate an LED when its respective key is pressed/released

        Parameters: color, speed, keyup/keydown

  · reactive-ripple
        Activate sourrounding LEDs when a key is pressed/released
        (sending a rippling wave over the keyboard)

        Parameters: color, speed, keyup/keydown

  · rain
        Like wave, but activate only a small number of random LEDs per
        row/column

        Parameters: color, speed, direction

  · gradient
        Create a static gradient (left to right)

        Parameters: color

  · fade
        Fade all LEDs simultaneously through a gradient

        Parameters: color, speed

  · And custom software effects, see below.


Parameters:
  · color=<color parameter>
        Sets the effect color:
        · rainbow (default)
              A rainbow
        · random[ized]
              Random colors, often on the rainbow spectrum
        · rgb:RRGGBB
              A single color by its HTML notation
        · gradient:{{RRGGBB@index,}}
              A gradient (up to ten colors), indices are in the [0, 100] range
              (only works for “gradient” and “fade”)
        · stdin (only for “all-keys”)
              Read all keys’ colors from stdin

  · speed=<0..100>
        Sets an effect’s speed.  Some effects may work with speeds above 100.

        (Default: 50)

  · direction=<right|left|down|up>
        Sets some effects’ target direction (i.e., “right” means from left to
        right, etc.)

        (Default: right)

  · keyup/keydown
        These choose the trigger event for the “reactive” events.

        (Default: keydown)


Software effects:
  · screen-capture
        Captures the screen (with ffmpeg) and mirrors it to the keyboard (scaled
        down to 18×6)");
}

fn strip_prefix<'a>(string: &'a str, prefix: &str) -> Option<&'a str> {
    if string.starts_with(prefix) {
        Some(string.split_at(prefix.len()).1)
    } else {
        None
    }
}

fn main() {
    /* Skip argv[0] */
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut profile = 1;

    /* Look for global switches before trying to open the keyboard */
    for arg in &argv {
        if !arg.starts_with("-") {
            continue;
        }

        let mut arg_split = arg.splitn(2, "=");

        match arg_split.next().unwrap() {
            "-h" | "-?" | "--help" => {
                print_usage();
                std::process::exit(0);
            }

            "-p" | "--profile" => {
                let profile_str =
                    match arg_split.next() {
                        Some(x) => x,
                        None => {
                            eprintln!("--profile requires an argument");
                            std::process::exit(1);
                        }
                    };

                profile =
                    match profile_str.parse::<u8>() {
                        Ok(x) => x,
                        Err(e) => {
                            eprintln!("{} is not a valid 8-bit integer: {}",
                                      profile_str, e);
                            std::process::exit(1);
                        }
                    };

                if profile < 1 || profile > 4 {
                    eprintln!("Profile index must be between 1 and 4 (incl.)");
                    std::process::exit(1);
                }
            }

            x => {
                eprintln!("Unrecognized switch “{}”", x);
                eprintln!("");
                print_usage();
                std::process::exit(1);
            }
        }
    }

    let mut kbd = Keyboard::new();
    kbd.set_profile(profile);

    for arg in &argv {
        if arg.starts_with("-") {
            continue;
        }

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
                        match Color::from_str(rgb) {
                            Ok(c) => cp = ColorParam::Color(c),

                            Err(msg) => {
                                eprintln!("{}", msg);
                                std::process::exit(1);
                            }
                        }
                    } else if let Some(gradient) = strip_prefix(c, "gradient:") {
                        match Gradient::from_str(gradient) {
                            Ok(g) => cp = ColorParam::Gradient(g),

                            Err(msg) => {
                                eprintln!("{}", msg);
                                std::process::exit(1);
                            }
                        }
                    } else if c == "stdin" {
                        match KeyMap::from_stdin() {
                            Ok(km) => cp = ColorParam::PerKey(km),

                            Err(msg) => {
                                eprintln!("{}", msg);
                                std::process::exit(1);
                            }
                        }
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

                        /* No need to continue parsing parameters for
                         * software effects */
                        match pkey {
                            "screen-capture" => break,
                            _ => (),
                        }
                    } else {
                        eprintln!("Unrecognized parameter key “{}”", pkey);
                        std::process::exit(1);
                    }
                }
            }
        }

        match effect.unwrap_or("all-keys") {
            "all-keys" => {
                match cp {
                    ColorParam::PerKey(km) => kbd.all_keys(&km),
                    cp => kbd.gradient(&cp),
                }
            }

            "pulse"             => kbd.pulse(&cp, speed),
            "wave"              => kbd.wave(&cp, speed, direction),
            "reactive"          => kbd.reactive(&cp, speed, keyup),
            "reactive-ripple"   => kbd.reactive_ripple(&cp, speed, keyup),
            "rain"              => kbd.rain(&cp, speed, direction),
            "gradient"          => kbd.gradient(&cp),
            "fade"              => kbd.fade(&cp, speed),

            "screen-capture" => {
                kbd.software_effect_start();
                screen_capture(&mut kbd, &arg);
                kbd.software_effect_end();
            }

            x => {
                eprintln!("Unrecognized effect “{}”", x);
                eprintln!("");
                print_usage();
                std::process::exit(1);
            }
        }
    }
}
