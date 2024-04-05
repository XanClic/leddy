use std::collections::HashMap;
use std::io::Read;
use std::process::{Command, Stdio};

use crate::check_superfluous_params;
use crate::keyboard::Keyboard;


fn isize_param(params: &mut HashMap<&str, &str>, name: &str)
    -> Result<Option<isize>, String>
{
    if let Some(val) = params.remove(name) {
        match val.parse() {
            Ok(x) => Ok(Some(x)),
            Err(e) => Err(format!("Invalid {} value “{}”: {}", name, val, e)),
        }
    } else {
        Ok(None)
    }
}

#[cfg(not(target_os = "windows"))]
fn xrandr_res() -> Result<(isize, isize), String> {
    let mut xrandr =
        match Command::new("xrandr")
                .arg("--query")
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
        {
            Ok(p) => p,

            Err(e) =>
                return Err(format!("Failed to launch xrandr: {}", e)),
        };

    let mut xrandr_output = String::new();
    xrandr.stdout.as_mut().unwrap().read_to_string(&mut xrandr_output).unwrap();

    xrandr.wait().unwrap();

    let xrandr_res = xrandr_output.split_once(", current ").unwrap().1;

    let mut xrandr_res_it = xrandr_res.splitn(2, " x ");
    let xrandr_w = xrandr_res_it.next().unwrap().parse().unwrap();

    let mut xrandr_res_it = xrandr_res_it.next().unwrap().splitn(2, ", ");
    let xrandr_h = xrandr_res_it.next().unwrap().parse().unwrap();

    Ok((xrandr_w, xrandr_h))
}

pub fn screen_capture(kbd: &Keyboard, mut params: HashMap<&str, &str>)
    -> Result<(), String>
{
    #[cfg(not(target_os = "windows"))]
    let (def_w, def_h) = xrandr_res()?;

    let ffmpeg_path = params.remove("ffmpeg-bin").unwrap_or("ffmpeg");
    let fps = isize_param(&mut params, "fps")?.unwrap_or(60);
    let x = isize_param(&mut params, "x")?;
    let y = isize_param(&mut params, "y")?;
    let w = isize_param(&mut params, "w")?;
    let h = isize_param(&mut params, "h")?;
    #[cfg(not(target_os = "windows"))]
    let display = params.remove("display").unwrap_or(":0");
    let scale_alg = params.remove("scale-algorithm").unwrap_or("area");

    check_superfluous_params(params)?;

    let mut ffmpeg_cmd = Command::new(ffmpeg_path);

    #[cfg(not(target_os = "windows"))]
    {
        ffmpeg_cmd.arg("-video_size")
                  .arg(format!("{}x{}",
                               w.unwrap_or(def_w), h.unwrap_or(def_h)));
    }

    #[cfg(target_os = "windows")]
    {
        if w.is_some() || h.is_some() {
            if w.is_none() || h.is_none() {
                return Err(String::from("You need to specify either both of w \
                                         and h, or neither"));
            }
            ffmpeg_cmd.arg("-video_size")
                      .arg(format!("{}x{}", w.unwrap(), h.unwrap()));
        }
    }

    ffmpeg_cmd.arg("-framerate").arg(format!("{}", fps));

    #[cfg(not(target_os = "windows"))]
    {
        ffmpeg_cmd.arg("-f").arg("x11grab")
                  .arg("-i")
                  .arg(format!("{}+{},{}",
                               display, x.unwrap_or(0), y.unwrap_or(0)));
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(xv) = x {
            ffmpeg_cmd.arg("-offset_x").arg(format!("{}", xv));
        }
        if let Some(yv) = y {
            ffmpeg_cmd.arg("-offset_y").arg(format!("{}", yv));
        }
        ffmpeg_cmd.arg("-f").arg("gdigrab")
                  .arg("-i").arg("desktop");
    }

    ffmpeg_cmd.arg("-vf").arg(format!("scale={}x6:sws_flags={}",
                                      kbd.width, scale_alg))
              .arg("-vcodec").arg("rawvideo")
              .arg("-f").arg("rawvideo")
              .arg("pipe:1")
              .stdin(Stdio::null())
              .stdout(Stdio::piped())
              .stderr(Stdio::null());

    let ffmpeg =
        match ffmpeg_cmd.spawn() {
            Ok(p) => p,

            Err(e) =>
                return Err(format!("Failed to launch ffmpeg: {}", e)),
        };

    let mut ffmpeg_stdout = ffmpeg.stdout.unwrap();
    let mut screen = vec![0u8; kbd.width * 6 * 4];
    let mut keys = vec![0u8; kbd.led_count * 3];

    loop {
        ffmpeg_stdout.read_exact(&mut screen).unwrap();

        for i in 0..(kbd.width * 6) {
            match kbd.ledmap[i] {
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
