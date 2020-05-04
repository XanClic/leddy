use std::collections::HashMap;
use std::io::Read;

use rustfft::FFTplanner;
use rustfft::num_complex::Complex;
use rustfft::num_traits::Zero;

use crate::check_superfluous_params;
use crate::keyboard::Keyboard;


const SAMPLES: usize = 2205;
const BUF_MSEC: usize = 1000 / (44100 / SAMPLES);


fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let c = v * s;
    let h1 = h * 6.0;
    let x = c * (1.0 - (h1 % 2.0 - 1.0).abs());

    let (r1, g1, b1) =
        if h1 < 1.0 {
            (c, x, 0.0)
        } else if h1 < 2.0 {
            (x, c, 0.0)
        } else if h1 < 3.0 {
            (0.0, c, x)
        } else if h1 < 4.0 {
            (0.0, x, c)
        } else if h1 < 5.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

    let m = v - c;
    (r1 + m, g1 + m, b1 + m)
}

pub fn sound_spectrum(kbd: &Keyboard, params: HashMap<&str, &str>)
    -> Result<(), String>
{
    check_superfluous_params(params)?;

    let samples = [0u16; SAMPLES];

    let mut buf = vec![Complex::<f32>::zero(); SAMPLES];
    let mut fft_res = vec![Complex::<f32>::zero(); SAMPLES];

    let mut fft_planner = FFTplanner::new(false);
    let fft = fft_planner.plan_fft(SAMPLES);

    /* FIXME: Use floating-point coordinates for better precision */
    let map: [[u8; 18]; 6] = [
        [1,    0,    7,   13,   19,   25,   31,   37,   43,   49, 0xff,   55,   67,   73,   79,   90,   93,   98],
        [2,    8,   14,   20,   26,   32,   38,   44,   50,   56,   61,   62,   68,   80, 0xff,   89,   94,   99],
        [3,    9,   15,   21,   27,   33,   39,   45,   51,   57,   63,   69,   75, 0xff,   81,   88,   95,   96],
        [4, 0xff,   10,   16,   22,   28,   34,   40,   46,   52,   58,   64,   70,   76,   82, 0xff, 0xff, 0xff],
        [5,   11,   17,   23,   29,   35,   41,   47,   53,   59,   65,   66, 0xff,   77, 0xff, 0xff,   87, 0xff],
        [6,   12, 0xff,   18, 0xff, 0xff,   36, 0xff, 0xff, 0xff,   60,   72, 0xff,   78,   83,   84,   85,   86],
    ];

    let mut keys = [0u8; 106 * 3];

    let mut scale = 0.0015f32;

    let mut last_lengths = [0.0f32; 5];
    let mut freqs = [0.0f32; 18];

    loop {
        std::io::stdin().read_exact(unsafe {
            std::slice::from_raw_parts_mut(samples.as_ptr() as *mut u8,
                                           SAMPLES * 2)
        }).unwrap();

        for i in 0..SAMPLES {
            buf[i].re = samples[i] as f32 / 32768.0;
            buf[i].im = 0.0;
        }

        fft.process(&mut buf, &mut fft_res);

        /* FIXME: .norm() is wrong (should be .re.abs()) */

        let intervals: [u8; 18] = [
            /* 40 – 80, 100 – 160, 180 – 240 */
            3, 4, 4,
            /* 260 – 280, 300 – 320, 340 – 380, 400 – 440, 460 – 500 */
            2, 2, 3, 3, 3,
            /* 520 – 580, 600 – 680, 700 – 780, 800 – 900, 920 – 1020 */
            4, 5, 5, 6, 6,
            /* 1040 – 1180, 1200 – 1360, 1380 – 1580, 1600 – 1820,
             * 1840 – 2060 */
            8, 9, 11, 12, 13,
        ];
        let mut si = 2;
        for i in 0..18 {
            let ei = si + intervals[i] as usize;

            freqs[i] = fft_res[si..ei].iter().fold(0.0, |c, x| c.max(x.norm()));

            si = ei;
        }

        let max_val = freqs.iter().fold(0.0f32, |c, x| c.max(*x));
        let scaled_max = max_val * scale;
        if scaled_max > 1.0 {
            scale /= scaled_max;
        } else {
            scale = (0.995 * scale + 0.005 * scale / scaled_max).min(0.003);
        }

        let (max_i, mut avg, mut max) =
            freqs[3..].iter().enumerate().fold((0, 0.0f32, 0.0f32),
                |(mi, ac, am), (xi, x)| {
                    if *x > am {
                        (xi + 3, ac + *x, *x)
                    } else {
                        (mi, ac + *x, am)
                    }
                });

        avg *= scale / 15.0;
        max *= scale;

        let saturation = ((16.0 / 15.0) * max - avg) / max.max(0.01);
        let color = ((max_i as isize - 3) as f32 / 22.0)
                        .max(0.0).min(4.0 / 6.0);
        let value = max.powf(2.0);

        let bg = hsv_to_rgb(color, saturation, value);
        let int_bg = ((bg.0 * 255.0 + 0.5) as u8,
                      (bg.1 * 255.0 + 0.5) as u8,
                      (bg.2 * 255.0 + 0.5) as u8);

        let base = freqs[..3].iter().fold(0.0f32, |m, x| m.max(*x)) * scale;
        let saturation = ((base - avg) / base.max(0.01)).max(0.0);
        let value = base.powf(2.0);
        let dir_bg = hsv_to_rgb(0.0, saturation, value);
        let int_dir_bg = ((dir_bg.0 * 255.0 + 0.5) as u8,
                          (dir_bg.1 * 255.0 + 0.5) as u8,
                          (dir_bg.2 * 255.0 + 0.5) as u8);

        for i in 0..(84 * 3) {
            keys[i] = 0;
        }
        for i in 84..88 {
            keys[i * 3 + 0] = int_dir_bg.0;
            keys[i * 3 + 1] = int_dir_bg.1;
            keys[i * 3 + 2] = int_dir_bg.2;
        }
        for i in 88..106 {
            keys[i * 3 + 0] = int_bg.0;
            keys[i * 3 + 1] = int_bg.1;
            keys[i * 3 + 2] = int_bg.2;
        }

        /* Low bass on space, alt */
        let intensity =
            (((freqs[0] * scale).powf(2.0) * 255.0).min(255.0) + 0.5) as u8;
        keys[18 * 3 + 0] = intensity;
        keys[36 * 3 + 0] = intensity;
        keys[60 * 3 + 0] = intensity;

        /* Mid bass on meta, fn, menu */
        let intensity =
            (((freqs[1] * scale).powf(2.0) * 255.0).min(255.0) + 0.5) as u8;
        keys[12 * 3 + 0] = intensity;
        keys[72 * 3 + 0] = intensity;
        keys[78 * 3 + 0] = intensity;

        /* High bass on control */
        let intensity =
            (((freqs[2] * scale).powf(2.0) * 255.0).min(255.0) + 0.5) as u8;
        keys[ 6 * 3 + 0] = intensity;
        keys[83 * 3 + 0] = intensity;

        for row_i in 0..5 {
            let fqib = 4 - row_i + 3;

            let raw_length = freqs[fqib + 0]
                        .max(freqs[fqib + 5])
                        .max(freqs[fqib + 10]);

            let rgb = ((freqs[fqib +  0] / raw_length).powf(2.0),
                       (freqs[fqib +  5] / raw_length).powf(2.0),
                       (freqs[fqib + 10] / raw_length).powf(2.0));

            let rgb_int = ((rgb.0 * 255.0 + 0.5) as u8,
                           (rgb.1 * 255.0 + 0.5) as u8,
                           (rgb.2 * 255.0 + 0.5) as u8);

            let length =
                if raw_length > last_lengths[row_i] {
                    raw_length
                } else {
                    (raw_length + last_lengths[row_i]) * 0.5
                };

            last_lengths[row_i] = length;

            let filled_bars =
                ((length * scale * 15.0).min(15.0) + 0.5) as usize;

            for j in 0..filled_bars {
                let ki = map[row_i][j] as usize;

                if ki != 0xff {
                    keys[ki * 3 + 0] = rgb_int.0;
                    keys[ki * 3 + 1] = rgb_int.1;
                    keys[ki * 3 + 2] = rgb_int.2;
                }
            }
        }

        kbd.all_keys_raw(&keys);
    }
}
