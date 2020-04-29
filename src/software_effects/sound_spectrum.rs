use std::collections::HashMap;
use std::io::Read;

use rustfft::FFTplanner;
use rustfft::num_complex::Complex;
use rustfft::num_traits::Zero;

use crate::check_superfluous_params;
use crate::keyboard::Keyboard;


const SAMPLES: usize = 2210;


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
        [1,    0,    7,   13,   19,   25,   31,   37,   43,   49,  103,   55,   67,   73,   79,   90,   93,   98],
        [2,    8,   14,   20,   26,   32,   38,   44,   50,   56,   61,   62,   68,   80, 0xff,   89,   94,   99],
        [3,    9,   15,   21,   27,   33,   39,   45,   51,   57,   63,   69,   75, 0xff,   81,   88,   95,   96],
        [4, 0xff,   10,   16,   22,   28,   34,   40,   46,   52,   58,   64,   70,   76,   82, 0xff, 0xff, 0xff],
        [5,   11,   17,   23,   29,   35,   41,   47,   53,   59,   65,   66, 0xff,   77, 0xff, 0xff,   87, 0xff],
        [6,   12, 0xff,   18, 0xff, 0xff,   36, 0xff, 0xff, 0xff,   60,   72, 0xff,   78,   83,   84,   85,   86],
    ];

    let mut keys = [0u8; 106 * 3];

    let scale = 0.01f32;

    let mut last_freqs = [0.0f32; 18];
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

        for i in 0..(106 * 3) {
            keys[i] = 0;
        }

        /* FIXME: .norm() is wrong (should be .re.abs()) */

        /*
         * 60 Hz intervals from 20 Hz, then 60 again, then 120, so we get:
         * - 20 to 380 Hz
         * - 380 to 740 Hz
         * - 740 to 1460 Hz
         */
        let mut si = 1;
        for i in 0..18 {
            let ei =
                if i < 12 {
                    si + 3
                } else {
                    si + 6
                };

            freqs[i] = fft_res[si..ei].iter().fold(0.0, |c, x| c + x.norm());

            si = ei;
        }

        for i in 0..18 {
            let color_i = i / 6;
            let row_i = 5 - i % 6;

            let val =
                if freqs[i] > last_freqs[i] {
                    freqs[i]
                } else {
                    (freqs[i] + last_freqs[i]) * 0.5
                };

            last_freqs[i] = val;

            let filled_bars = ((val * scale).min(18.0) + 0.5) as usize;

            for j in 0..filled_bars {
                let ki = map[row_i][j] as usize;

                if ki != 0xff {
                    keys[ki * 3 + color_i] = 0xff;
                }
            }
        }

        kbd.all_keys_raw(&keys);
    }
}
