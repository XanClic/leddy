use std::collections::HashMap;
use std::io::Read;
use std::process::{Command, Stdio};

use crate::check_superfluous_params;
use crate::keyboard::Keyboard;


pub fn screen_capture(kbd: &Keyboard, params: HashMap<&str, &str>)
    -> Result<(), String>
{
    check_superfluous_params(params)?;

    let mut xrandr =
        match Command::new("xrandr")
                .arg("--query")
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
        {
            Ok(p) => p,

            Err(e) => {
                eprintln!("Failed to launch xrandr: {}", e);
                std::process::exit(1);
            }
        };

    let mut xrandr_output = String::new();
    xrandr.stdout.as_mut().unwrap().read_to_string(&mut xrandr_output).unwrap();

    xrandr.wait().unwrap();

    let xrandr_res =
        xrandr_output.splitn(2, ", current ").skip(1).next().unwrap();

    let mut xrandr_res_it = xrandr_res.splitn(2, " x ");
    let xrandr_w: usize = xrandr_res_it.next().unwrap().parse().unwrap();

    let mut xrandr_res_it = xrandr_res_it.next().unwrap().splitn(2, ", ");
    let xrandr_h: usize = xrandr_res_it.next().unwrap().parse().unwrap();

    let ffmpeg =
        match Command::new("ffmpeg")
                .arg("-video_size").arg(format!("{}x{}", xrandr_w, xrandr_h))
                .arg("-framerate").arg("60")
                .arg("-f").arg("x11grab")
                .arg("-i").arg(":0.0+0,0")
                .arg("-vf").arg("scale=18x6:sws_flags=area")
                .arg("-vcodec").arg("rawvideo")
                .arg("-f").arg("rawvideo")
                .arg("pipe:1")
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
        {
            Ok(p) => p,

            Err(e) => {
                eprintln!("Failed to launch ffmpeg: {}", e);
                std::process::exit(1);
            }
        };

    let mut ffmpeg_stdout = ffmpeg.stdout.unwrap();
    let mut screen = [0u8; 18 * 6 * 4];
    let mut keys = [0u8; 106 * 3];

    let map: [u8; 18 * 6] = [
           1,    0,    7,   13,   19,   25,   31,   37,   43,   49, 0xff,   55,   67,   73,   79,   90,   93,   98,
           2,    8,   14,   20,   26,   32,   38,   44,   50,   56,   61,   62,   68,   80, 0xff,   89,   94,   99,
           3,    9,   15,   21,   27,   33,   39,   45,   51,   57,   63,   69,   75, 0xff,   81,   88,   95,   96,
           4, 0xff,   10,   16,   22,   28,   34,   40,   46,   52,   58,   64,   70,   76,   82, 0xff, 0xff, 0xff,
           5,   11,   17,   23,   29,   35,   41,   47,   53,   59,   65,   66, 0xff,   77, 0xff, 0xff,   87, 0xff,
           6,   12, 0xff,   18, 0xff, 0xff,   36, 0xff, 0xff, 0xff,   60,   72, 0xff,   78,   83,   84,   85,   86,
    ];

    loop {
        ffmpeg_stdout.read_exact(&mut screen).unwrap();

        for i in 0..(18 * 6) {
            match map[i] {
                0xff => (),
                m => {
                    let m_base = m as usize * 3;

                    keys[m_base + 0] = screen[i * 4 + 2];
                    keys[m_base + 1] = screen[i * 4 + 1];
                    keys[m_base + 2] = screen[i * 4 + 0];
                }
            }
        }

        kbd.all_keys_raw(&keys);
    }
}
