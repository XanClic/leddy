use rand::seq::SliceRandom;


pub type Color = (u8, u8, u8);

pub trait ColorMethods: std::marker::Sized {
    const RED: Self;
    const GREEN: Self;
    const BLUE: Self;

    const YELLOW: Self;
    const CYAN: Self;
    const MAGENTA: Self;

    fn from_str(s: &str) -> Result<Self, String>;
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

    fn from_str(s: &str) -> Result<Color, String> {
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
            return Err(format!("{} is not an RRGGBB value", s));
        }

        let ls = s.to_ascii_lowercase().bytes().collect::<Vec<u8>>();
        for c in &ls {
            if !((*c >= 48 && *c <= 57) || (*c >= 97 && *c <= 102)) {
                return Err(format!("{} is not an RRGGBB value", s));
            }
        }

        Ok(((hex_nibble(ls[0]) << 4) | hex_nibble(ls[1]),
            (hex_nibble(ls[2]) << 4) | hex_nibble(ls[3]),
            (hex_nibble(ls[4]) << 4) | hex_nibble(ls[5])))
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
    pub fn from_str(s: &str) -> Result<Gradient, String> {
        let mut proto_vec = Vec::<(Color, Option<u8>)>::new();

        for gci in s.split(',') {
            let mut gcis = gci.splitn(2, '@');
            let cols = gcis.next().unwrap();

            let coli =
                if let Some(is) = gcis.next() {
                    let val = is.parse().unwrap();
                    if val > 100 {
                        return Err(String::from("Gradient positions must not \
                                                 exceed 100"));
                    }
                    Some(val)
                } else {
                    None
                };

            let col = Color::from_str(cols)?;
            proto_vec.push((col, coli));
        }

        if proto_vec.len() < 1 {
            return Err(String::from("Gradients must have at least one color"));
        } else if proto_vec.len() > 10 {
            return Err(String::from("Gradients cannot have more than ten \
                                     colors"));
        }

        if let Some(x) = proto_vec.first_mut() {
            if x.1.is_none() {
                x.1 = Some(0);
            }
        }
        if let Some(x) = proto_vec.last_mut() {
            if x.1.is_none() {
                x.1 = Some(100);
            }
        }

        let mut base_pos = 0;
        let mut diff = 0;
        let mut diff_i = 0;
        let mut in_diff_i = 0;

        for i in 0..proto_vec.len() {
            if let Some(pos) = proto_vec[i].1 {
                base_pos = pos;
                diff_i = 0;
                in_diff_i = 0;
            } else {
                if in_diff_i == 0 {
                    let mut j = i + 1;
                    /* We did set the last position to Some(100) */
                    while proto_vec[j].1.is_none() {
                        j += 1;
                    }
                    diff = proto_vec[j].1.unwrap() as isize - base_pos as isize;
                    diff_i = (j - i + 1) as isize;
                }
                in_diff_i += 1;

                let itpl_pos = base_pos +
                               (in_diff_i * diff / diff_i) as u8;

                proto_vec[i].1 = Some(itpl_pos);
            }
        }

        let mut gradient = Gradient {
            colors: proto_vec.iter().map(|cp| (cp.0, cp.1.unwrap())).collect(),
        };

        gradient.colors.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        Ok(gradient)
    }

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
