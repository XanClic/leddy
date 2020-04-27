use std::collections::HashMap;

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

fn parse_color(color_param: &str) -> Result<ColorParam, String> {
    if color_param == "rainbow" {
        Ok(ColorParam::Rainbow)
    } else if color_param == "random" || color_param == "randomized" {
        Ok(ColorParam::Randomized)
    } else if let Some(rgb) = strip_prefix(color_param, "rgb:") {
        Ok(ColorParam::Color(Color::from_str(rgb)?))
    } else if let Some(gradient) = strip_prefix(color_param, "gradient:") {
        Ok(ColorParam::Gradient(Gradient::from_str(gradient)?))
    } else if color_param == "stdin" {
        Ok(ColorParam::PerKey(KeyMap::from_stdin()?))
    } else {
        Err(format!("Unrecognized color parameter “{}”", color_param))
    }
}

fn parse_speed(speed_param: &str) -> Result<u8, String> {
    match speed_param.parse() {
        Ok(x) => Ok(x),
        Err(e) => Err(format!("{} is not an 8-bit unsigned integer: {}",
                              speed_param, e)),
    }
}

fn parse_direction(dir_param: &str) -> Result<Direction, String> {
    match dir_param {
        "right" => Ok(Direction::Right),
        "left"  => Ok(Direction::Left),
        "down"  => Ok(Direction::Down),
        "up"    => Ok(Direction::Up),

        x => Err(format!("Invalid direction “{}”", x)),
    }
}

fn parse_keyup(up_param: Option<&str>, down_param: Option<&str>)
    -> Result<bool, String>
{
    if up_param.is_some() && down_param.is_some() {
        Err(String::from("Cannot give both keyup and keydown"))
    } else if up_param.is_some() {
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn check_superfluous_params(params: HashMap<&str, &str>)
    -> Result<(), String>
{
    fn string_list(mut cum: Option<String>, k: &&str) -> Option<String> {
        if let Some(mut cum_uwu) = cum.take() {
            cum_uwu.push_str(format!(", “{}”", k).as_str());
            Some(cum_uwu)
        } else {
            Some(format!("“{}”", k))
        }
    }

    if let Some(sup_params_str) = params.keys().fold(None, string_list) {
        Err(format!("Superfluous parameters: {}", sup_params_str))
    } else {
        Ok(())
    }
}


fn do_all_keys(kbd: &Keyboard, mut params: HashMap<&str, &str>)
    -> Result<(), String>
{
    let cp = parse_color(params.remove("color").unwrap_or("rainbow"))?;

    check_superfluous_params(params)?;

    match cp {
        ColorParam::PerKey(km) => kbd.all_keys(&km),
        _ => kbd.gradient(cp),
    }

    Ok(())
}

fn do_pulse(kbd: &Keyboard, mut params: HashMap<&str, &str>)
    -> Result<(), String>
{
    let cp = parse_color(params.remove("color").unwrap_or("rainbow"))?;
    let speed = parse_speed(params.remove("speed").unwrap_or("50"))?;

    check_superfluous_params(params)?;

    kbd.pulse(cp, speed);

    Ok(())
}

fn do_wave(kbd: &Keyboard, mut params: HashMap<&str, &str>)
    -> Result<(), String>
{
    let cp = parse_color(params.remove("color").unwrap_or("rainbow"))?;
    let speed = parse_speed(params.remove("speed").unwrap_or("50"))?;
    let dir = parse_direction(params.remove("direction").unwrap_or("right"))?;

    check_superfluous_params(params)?;

    kbd.wave(cp, speed, dir);

    Ok(())
}

fn do_reactive(kbd: &Keyboard, mut params: HashMap<&str, &str>)
    -> Result<(), String>
{
    let cp = parse_color(params.remove("color").unwrap_or("rainbow"))?;
    let speed = parse_speed(params.remove("speed").unwrap_or("50"))?;
    let keyup = parse_keyup(params.remove("keyup"), params.remove("keydown"))?;

    check_superfluous_params(params)?;

    kbd.reactive(cp, speed, keyup);

    Ok(())
}

fn do_reactive_ripple(kbd: &Keyboard, mut params: HashMap<&str, &str>)
    -> Result<(), String>
{
    let cp = parse_color(params.remove("color").unwrap_or("rainbow"))?;
    let speed = parse_speed(params.remove("speed").unwrap_or("50"))?;
    let keyup = parse_keyup(params.remove("keyup"), params.remove("keydown"))?;

    check_superfluous_params(params)?;

    kbd.reactive_ripple(cp, speed, keyup);

    Ok(())
}

fn do_rain(kbd: &Keyboard, mut params: HashMap<&str, &str>)
    -> Result<(), String>
{
    let cp = parse_color(params.remove("color").unwrap_or("randomized"))?;
    let speed = parse_speed(params.remove("speed").unwrap_or("50"))?;
    let dir = parse_direction(params.remove("direction").unwrap_or("right"))?;

    check_superfluous_params(params)?;

    kbd.rain(cp, speed, dir);

    Ok(())
}

fn do_gradient(kbd: &Keyboard, mut params: HashMap<&str, &str>)
    -> Result<(), String>
{
    let cp = parse_color(params.remove("color").unwrap_or("rainbow"))?;

    check_superfluous_params(params)?;

    kbd.gradient(cp);

    Ok(())
}

fn do_fade(kbd: &Keyboard, mut params: HashMap<&str, &str>)
    -> Result<(), String>
{
    let cp = parse_color(params.remove("color").unwrap_or("rainbow"))?;
    let speed = parse_speed(params.remove("speed").unwrap_or("50"))?;

    check_superfluous_params(params)?;

    kbd.fade(cp, speed);

    Ok(())
}


fn do_software_effect(kbd: &mut Keyboard, params: HashMap<&str, &str>,
                      efn: fn(&Keyboard, HashMap<&str, &str>)
                               -> Result<(), String>)
    -> Result<(), String>
{
    kbd.software_effect_start();
    let res = efn(kbd, params);
    kbd.software_effect_end();
    res
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

        let mut effect = HashMap::<&str, &str>::new();

        for param in arg.split('/') {
            let mut ps = param.splitn(2, '=');
            let pkey = ps.next().unwrap();

            let old_val_opt =
                match ps.next() {
                    Some(pval) => effect.insert(pkey, pval),
                    None => {
                        if effect.contains_key("name") {
                            effect.insert(pkey, "")
                        } else {
                            effect.insert("name", pkey)
                        }
                    }
                };

            if let Some(old_val) = old_val_opt {
                eprintln!("Effect parameter “{}” already set to “{}”",
                          pkey, old_val);
                std::process::exit(1);
            }
        }

        let result =
            match effect.remove("name").unwrap_or("all-keys") {
                "all-keys"          => do_all_keys(&kbd, effect),
                "pulse"             => do_pulse(&kbd, effect),
                "wave"              => do_wave(&kbd, effect),
                "reactive"          => do_reactive(&kbd, effect),
                "reactive-ripple"   => do_reactive_ripple(&kbd, effect),
                "rain"              => do_rain(&kbd, effect),
                "gradient"          => do_gradient(&kbd, effect),
                "fade"              => do_fade(&kbd, effect),

                "screen-capture"    => do_software_effect(&mut kbd, effect,
                                                          screen_capture),

                x => Err(format!("Unrecognized effect “{}”", x)),
            };

        if let Err(e) = result {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}
