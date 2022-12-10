use std::collections::HashMap;
use std::io::Read;

use rustfft::FFTplanner;
use rustfft::num_complex::Complex;
use rustfft::num_traits::Zero;

use crate::check_superfluous_params;
use crate::keyboard::Keyboard;


const SAMPLES: usize = 2205;
const SAMPLES_MSECS: usize = 1000 / (44100 / SAMPLES);
const SAMPLES_USED: usize = 200; /* up to 4 kHz */
const INACTIVITY_MSECS: usize = 5000;


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
    let mut fft_vals = vec![0f32; SAMPLES_USED];

    let mut fft_planner = FFTplanner::new(false);
    let fft = fft_planner.plan_fft(SAMPLES);

    let peak_keys = [
        if kbd.mini {
            vec![
                Keyboard::LEFT,
                Keyboard::DOWN,
                Keyboard::RIGHT,
                Keyboard::UP,
            ]
        } else {
            vec![
                Keyboard::LEFT,
                Keyboard::DOWN,
                Keyboard::RIGHT,
                Keyboard::UP,
                Keyboard::DELETE,
                Keyboard::INSERT,
                Keyboard::PRINT,
                Keyboard::SCROLL_LOCK,
                Keyboard::HOME,
                Keyboard::END,
                Keyboard::PAGE_DOWN,
                Keyboard::PAUSE,
                Keyboard::PAGE_UP,
            ]
        },

        if kbd.mini {
            vec![
                Keyboard::DELETE,
                Keyboard::INSERT,
                Keyboard::PRINT,
                Keyboard::MUTE_MIC,
                Keyboard::MUTE_SPEAKER,
                Keyboard::SCROLL_LOCK,
                Keyboard::HOME,
                Keyboard::END,
                Keyboard::PAGE_DOWN,
                Keyboard::GAMING_MODE,
                Keyboard::PAUSE,
                Keyboard::PAGE_UP,
                Keyboard::MINI_SIG_PLATE,
            ]
        } else {
            vec![
                Keyboard::NUM_LOCK,
                Keyboard::NUM_7,
                Keyboard::NUM_4,
                Keyboard::NUM_1,
                Keyboard::NUM_0,
                Keyboard::NUM_2,
                Keyboard::NUM_5,
                Keyboard::NUM_8,
                Keyboard::NUM_SLASH,
                Keyboard::NUM_ASTERISK,
                Keyboard::NUM_9,
                Keyboard::NUM_6,
                Keyboard::NUM_3,
                Keyboard::NUM_DECIMAL,
                Keyboard::NUM_ENTER,
                Keyboard::NUM_PLUS,
                Keyboard::NUM_MINUS,
                Keyboard::VOLUME_KNOB,
                Keyboard::FULL_SIG_PLATE,
            ]
        },
    ];

    let mut keys = vec![0u8; kbd.led_count * 3];

    let mut scale = 0.0015f32;

    let mut last_lengths = [0.0f32; 5];
    let mut freqs = [0.0f32; 18];

    let mut inactivity_msecs = 0usize;

    loop {
        for i in 0..keys.len() {
            keys[i] = 0;
        }

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
        for i in 0..SAMPLES_USED {
            fft_vals[i] = fft_res[i].norm();
        }

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
        let mut freqs_max = 0.01f32; /* avoid later division by zero */
        for i in 0..18 {
            let ei = si + intervals[i] as usize;

            freqs[i] = fft_vals[si..ei].iter().fold(0.0, |c, x| c.max(*x));
            freqs_max = freqs_max.max(freqs[i]);

            si = ei;
        }

        if freqs_max <= 0.01f32 {
            let was_below = inactivity_msecs < INACTIVITY_MSECS;

            inactivity_msecs = inactivity_msecs.saturating_add(SAMPLES_MSECS);
            if inactivity_msecs >= INACTIVITY_MSECS {
                if was_below {
                    kbd.refresh_profile();
                }
                continue;
            }
        } else {
            inactivity_msecs = 0;
        }

        let avgs = [
            freqs[0..3].iter().fold(0.0f32, |c, x| c + x) * (1.0 / 3.0),
            freqs[3..18].iter().fold(0.0f32, |c, x| c + x) * (1.0 / 15.0),
        ];

        let scaled_max = freqs_max * scale;
        if scaled_max > 1.0 {
            scale /= scaled_max;
        } else {
            scale = (0.995 * scale + 0.005 * scale / scaled_max).min(0.003);
        }

        let peaks: [(usize, f32); 2] = [
            /* 40 to 240 Hz */
            (0, freqs[0..3].iter().fold(0.0f32, |c, x| c.max(*x))),

            /* 260 Hz up to 4 kHz */
            fft_vals[13..200].iter().zip(13..200)
                .fold((0, 0.0), |(max_i, max_v), (v, i)| {
                    if *v > max_v {
                        (i, *v)
                    } else {
                        (max_i, max_v)
                    }
                }),
        ];

        for i in 0..2 {
            let sat =
                if i == 0 {
                    1.0
                } else {
                    ((peaks[i].1 - (14.0 / 15.0) * avgs[i])
                     / freqs_max).min(1.0)
                };

            /* Red until 260 Hz, green at 680 Hz, capped at violet
             * (log2 scale) */
            let col = (((peaks[i].0 as f32).log2() - 3.7) * 0.2402466743058456)
                        .max(0.0).min(5.0 / 6.0);
            let val = (peaks[i].1 * scale).powf(2.0).min(1.0);

            let rgb = hsv_to_rgb(col, sat, val);
            let int_rgb = ((rgb.0 * 255.0 + 0.5) as u8,
                           (rgb.1 * 255.0 + 0.5) as u8,
                           (rgb.2 * 255.0 + 0.5) as u8);

            for key_i in &peak_keys[i] {
                keys[key_i * 3 + 0] = int_rgb.0;
                keys[key_i * 3 + 1] = int_rgb.1;
                keys[key_i * 3 + 2] = int_rgb.2;
            }
        }

        /* Low bass on space, alt */
        let intensity =
            (((freqs[0] * scale).powf(2.0) * 255.0).min(255.0) + 0.5) as u8;
        keys[Keyboard::LALT  * 3 + 0] = intensity;
        keys[Keyboard::SPACE * 3 + 0] = intensity;
        keys[Keyboard::RALT  * 3 + 0] = intensity;

        /* Mid bass on meta, fn, menu */
        let intensity =
            (((freqs[1] * scale).powf(2.0) * 255.0).min(255.0) + 0.5) as u8;
        keys[Keyboard::META * 3 + 0] = intensity;
        keys[Keyboard::FN   * 3 + 0] = intensity;
        keys[Keyboard::MENU * 3 + 0] = intensity;

        /* High bass on control */
        let intensity =
            (((freqs[2] * scale).powf(2.0) * 255.0).min(255.0) + 0.5) as u8;
        keys[Keyboard::LCONTROL * 3 + 0] = intensity;
        keys[Keyboard::RCONTROL * 3 + 0] = intensity;

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
                let ki = kbd.ledmap[row_i * kbd.width + j] as usize;

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
