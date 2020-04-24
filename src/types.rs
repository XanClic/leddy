use rand::seq::SliceRandom;


pub type Color = (u8, u8, u8);

pub trait ColorMethods {
    const RED: Self;
    const GREEN: Self;
    const BLUE: Self;

    const YELLOW: Self;
    const CYAN: Self;
    const MAGENTA: Self;

    fn from_str(s: &str) -> Self;
}

pub struct Gradient {
    pub colors: Vec<(Color, u8)>,
}

pub enum ColorParam {
    Color(Color),
    Rainbow,
    Randomized,
    Gradient(Gradient),
}

pub enum Direction {
    Right = 1,
    Left = 2,
    Down = 3,
    Up = 4,
}


impl ColorMethods for Color {
    const RED: Color        = (0xff, 0x00, 0x00);
    const GREEN: Color      = (0x00, 0xff, 0x00);
    const BLUE: Color       = (0x00, 0x00, 0xff);

    const YELLOW: Color     = (0xff, 0xff, 0x00);
    const CYAN: Color       = (0x00, 0xff, 0xff);
    const MAGENTA: Color    = (0xff, 0x00, 0xff);

    fn from_str(s: &str) -> Color {
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
}


impl ColorParam {
    pub fn mode(&self) -> u8 {
        match self {
            ColorParam::Color(_) => 0,
            ColorParam::Rainbow => 1,
            ColorParam::Randomized => 2,
            ColorParam::Gradient(_) => 3,
        }
    }

    pub fn rgb(&self) -> Color {
        match self {
            ColorParam::Color(c) => *c,
            ColorParam::Rainbow => (0, 0, 0),
            ColorParam::Randomized => (0, 0, 0),
            ColorParam::Gradient(g) => g.colors[0].0,
        }
    }

    pub fn gradient(&self) -> Gradient {
        match self {
            ColorParam::Color(c) => {
                Gradient {
                    colors: vec![(*c, 0), (*c, 100)]
                }
            }

            ColorParam::Rainbow => {
                Gradient {
                    colors: vec![
                        (Color::RED,         0),
                        (Color::YELLOW,     20),
                        (Color::GREEN,      40),
                        (Color::CYAN,       60),
                        (Color::BLUE,       80),
                        (Color::MAGENTA,   100)
                    ]
                }
            }

            ColorParam::Randomized => {
                let mut colors = [
                    Color::RED,
                    Color::YELLOW,
                    Color::GREEN,
                    Color::CYAN,
                    Color::BLUE,
                    Color::MAGENTA,
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
    pub fn serialize(&self, to: &mut [u8]) {
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
