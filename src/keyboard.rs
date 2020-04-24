use hidapi::{HidApi, HidDevice};
use crate::types::{Color, ColorParam, Direction};


pub struct Keyboard {
    dev: HidDevice,
    make_changes_permanent: bool,
}


impl Keyboard {
    pub fn new() -> Self {
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

    pub fn send_req(&self, raw_data: &[u8]) {
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
    pub fn all_keys(&self, keys: &[Color; 106]) {
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

    pub fn pulse(&self, cp: &ColorParam, speed: u8) {
        let rgb = cp.rgb();

        self.send_req(&[0x0f, 0x06,
                      cp.mode(),
                      rgb.0, rgb.1, rgb.2,
                      speed]);
    }

    pub fn wave(&self, cp: &ColorParam, speed: u8, direction: Direction) {
        let rgb = cp.rgb();

        self.send_req(&[0x0f, 0x07,
                      cp.mode(),
                      rgb.0, rgb.1, rgb.2,
                      speed,
                      direction as u8]);
    }

    pub fn reactive(&self, cp: &ColorParam, speed: u8, keyup: bool) {
        let rgb = cp.rgb();

        self.send_req(&[0x0f, 0x09,
                      cp.mode(),
                      rgb.0, rgb.1, rgb.2,
                      speed,
                      !keyup as u8]);
    }

    pub fn reactive_ripple(&self, cp: &ColorParam, speed: u8, keyup: bool) {
        let rgb = cp.rgb();

        self.send_req(&[0x0f, 0x0a,
                      cp.mode(),
                      rgb.0, rgb.1, rgb.2,
                      speed,
                      !keyup as u8]);
    }

    pub fn rain(&self, cp: &ColorParam, speed: u8, direction: Direction) {
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

    pub fn gradient(&self, cp: &ColorParam) {
        let mut req = [0u8; 43];

        req[0] = 0x0f;
        req[1] = 0x0c;
        cp.gradient().serialize(&mut req[2..43]);
        self.send_req(&req);
    }

    pub fn fade(&self, cp: &ColorParam, speed: u8) {
        let mut req = [0u8; 45];

        req[0] = 0x0f;
        req[1] = 0x0d;
        req[2] = cp.mode();
        cp.gradient().serialize(&mut req[3..44]);
        req[44] = speed;

        self.send_req(&req);
    }
}

