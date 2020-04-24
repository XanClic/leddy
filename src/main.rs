use hidapi::{HidApi, HidDevice};
use rand::seq::SliceRandom;


type Color = (u8, u8, u8);

struct Gradient {
    colors: Vec<(Color, u8)>,
}

enum ColorParam {
    Color(Color),
    Rainbow,
    Randomized,
    Gradient(Gradient),
}

enum Direction {
    Right = 1,
    Left = 2,
    Down = 3,
    Up = 4,
}

struct Keyboard {
    dev: HidDevice,
    make_changes_permanent: bool,
}


fn color_from_str(s: &str) -> Color {
    fn hex_nibble(c: u8) -> u8 {
        if c >= 48 && c <= 57 {
            c - 48
        } else if c >= 97 && c <= 102 {
            c - 97 + 10
        } else {
            unreachable!();
        }
    }

    if s.len() != 6 {
        panic!("{} is not an RRGGBB value", s);
    }

    let ls = s.to_ascii_lowercase().bytes().collect::<Vec<u8>>();
    for c in &ls {
        if !((*c >= 48 && *c <= 57) || (*c >= 97 && *c <= 102)) {
            panic!("{} is not an RRGGBB value", s);
        }
    }

    ((hex_nibble(ls[0]) << 4) | hex_nibble(ls[1]),
     (hex_nibble(ls[2]) << 4) | hex_nibble(ls[3]),
     (hex_nibble(ls[4]) << 4) | hex_nibble(ls[5]))
}

impl ColorParam {
    fn mode(&self) -> u8 {
        match self {
            ColorParam::Color(_) => 0,
            ColorParam::Rainbow => 1,
            ColorParam::Randomized => 2,
            ColorParam::Gradient(_) => 3,
        }
    }

    fn rgb(&self) -> Color {
        match self {
            ColorParam::Color(c) => *c,
            ColorParam::Rainbow => (0, 0, 0),
            ColorParam::Randomized => (0, 0, 0),
            ColorParam::Gradient(g) => g.colors[0].0,
        }
    }

    fn gradient(&self) -> Gradient {
        match self {
            ColorParam::Color(c) => {
                Gradient {
                    colors: vec![(*c, 0), (*c, 100)]
                }
            }

            ColorParam::Rainbow => {
                Gradient {
                    colors: vec![
                        ((0xff, 0x00, 0x00),   0),
                        ((0xff, 0xff, 0x00),  20),
                        ((0x00, 0xff, 0x00),  40),
                        ((0x00, 0xff, 0xff),  60),
                        ((0x00, 0x00, 0xff),  80),
                        ((0xff, 0x00, 0xff), 100)
                    ]
                }
            }

            ColorParam::Randomized => {
                let mut colors = [
                    (0xff, 0x00, 0x00),
                    (0xff, 0xff, 0x00),
                    (0x00, 0xff, 0x00),
                    (0x00, 0xff, 0xff),
                    (0x00, 0x00, 0xff),
                    (0xff, 0x00, 0xff)
                ];

                colors.shuffle(&mut rand::thread_rng());

                Gradient {
                    colors: vec![
                        (colors[0],   0),
                        (colors[1],  20),
                        (colors[2],  40),
                        (colors[3],  60),
                        (colors[4],  80),
                        (colors[5], 100)
                    ]
                }
            }

            ColorParam::Gradient(g) => {
                Gradient {
                    colors: g.colors.clone()
                }
            }
        }
    }
}

impl Gradient {
    fn serialize(&self, to: &mut [u8]) {
        let len = self.colors.len();

        assert!(len > 0 && len <= 10);

        to[0] = len as u8;

        let mut i = 0;

        for color in &self.colors {
            to[i * 4 + 1] = (color.0).0;
            to[i * 4 + 2] = (color.0).1;
            to[i * 4 + 3] = (color.0).2;
            to[i * 4 + 4] = color.1;
            i += 1;
        }

        while i < 10 {
            to[i * 4 + 1] = 0;
            to[i * 4 + 2] = 0;
            to[i * 4 + 3] = 0;
            to[i * 4 + 4] = 0;
            i += 1;
        }
    }
}


impl Keyboard {
    fn new() -> Self {
        let hidapi = HidApi::new().unwrap();

        let mut dev_opt = None;
        for dev in hidapi.device_list() {
            if dev.vendor_id() == 0x2f0e && dev.product_id() == 0x0102 &&
               dev.interface_number() == 1
            {
                dev_opt = Some(dev);
                break;
            }
        }

        let dev = match dev_opt.unwrap().open_device(&hidapi) {
            Ok(x) => x,
            Err(e) => {
                eprintln!("Failed to open HID device: {}\n\
                           Check whether you have the required access rights.",
                          e);
                std::process::exit(1);
            }
        };

        Keyboard {
            dev: dev,
            make_changes_permanent: true,
        }
    }

    fn send_req(&self, raw_data: &[u8]) {
        let len = raw_data.len();
        let mut ofs = 0;

        while ofs < len {
            let mut data = [0u8; 65];

            data[0] = 0x00;

            data[1] = raw_data[0];

            data[2] = len as u8;
            data[3] = (len >> 8) as u8;
            data[4] = (len >> 16) as u8;

            data[5] = ofs as u8;
            data[6] = (ofs >> 8) as u8;
            data[7] = (ofs >> 16) as u8;

            for i in ofs..std::cmp::min(ofs + 57, len) {
                data[i - ofs + 8] = raw_data[i];
            }

            self.dev.write(&data).unwrap();

            ofs += 57;
        }

        if raw_data[0] == 0x0f && self.make_changes_permanent {
            /* Make change permanent */
            self.send_req(&[0x04, raw_data[1] - 1]);
        }
    }

    #[allow(unused)]
    fn all_keys(&self, keys: &[Color; 106]) {
        let mut cmd = [0u8; 2 + 106 * 3];

        cmd[0] = 0x0f;
        cmd[1] = 0x03;

        for i in 0..106 {
            cmd[i * 3 + 2] = keys[i].0;
            cmd[i * 3 + 3] = keys[i].1;
            cmd[i * 3 + 4] = keys[i].2;
        }

        self.send_req(&cmd);
    }

    fn pulse(&self, cp: &ColorParam, speed: u8) {
        let rgb = cp.rgb();

        self.send_req(&[0x0f, 0x06,
                      cp.mode(),
                      rgb.0, rgb.1, rgb.2,
                      speed]);
    }

    fn wave(&self, cp: &ColorParam, speed: u8, direction: Direction) {
        let rgb = cp.rgb();

        self.send_req(&[0x0f, 0x07,
                      cp.mode(),
                      rgb.0, rgb.1, rgb.2,
                      speed,
                      direction as u8]);
    }

    fn reactive(&self, cp: &ColorParam, speed: u8, keyup: bool) {
        let rgb = cp.rgb();

        self.send_req(&[0x0f, 0x09,
                      cp.mode(),
                      rgb.0, rgb.1, rgb.2,
                      speed,
                      !keyup as u8]);
    }

    fn reactive_ripple(&self, cp: &ColorParam, speed: u8, keyup: bool) {
        let rgb = cp.rgb();

        self.send_req(&[0x0f, 0x0a,
                      cp.mode(),
                      rgb.0, rgb.1, rgb.2,
                      speed,
                      !keyup as u8]);
    }

    fn rain(&self, cp: &ColorParam, speed: u8, direction: Direction) {
        let rgb = cp.rgb();

        /* This effect does not support rainbow mode */
        let mode =
            match cp {
                ColorParam::Rainbow => (ColorParam::Randomized).mode(),
                _ => cp.mode()
            };

        self.send_req(&[0x0f, 0x0b,
                      mode,
                      rgb.0, rgb.1, rgb.2,
                      speed,
                      direction as u8]);
    }

    fn gradient(&self, cp: &ColorParam) {
        let mut req = [0u8; 43];

        req[0] = 0x0f;
        req[1] = 0x0c;
        cp.gradient().serialize(&mut req[2..43]);
        self.send_req(&req);
    }

    fn fade(&self, cp: &ColorParam, speed: u8) {
        let mut req = [0u8; 45];

        req[0] = 0x0f;
        req[1] = 0x0d;
        req[2] = cp.mode();
        cp.gradient().serialize(&mut req[3..44]);
        req[44] = speed;

        self.send_req(&req);
    }
}

fn print_usage() {
    eprintln!("Usage: leddy <[effect/]{{parameters...}}>
Parameters are separated by slashes.

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
    let kbd = Keyboard::new();

    let argv: Vec<String> = std::env::args().collect();

    let mut arg_iter = argv.iter();
    arg_iter.next(); /* skip argv[0] */

    for arg in arg_iter {
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
                        cp = ColorParam::Color(color_from_str(rgb));
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

                                    (color_from_str(cols), coli)
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
