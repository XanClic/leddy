use hidapi::{HidApi, HidDevice};
use crate::types::{ColorParam, Direction, KeyMap};


pub struct Keyboard {
    dev: HidDevice,
    make_changes_permanent: bool,
}


impl Keyboard {
    const COLOR_TEMP: [u8; 1] = [0x0f];
    const COLOR_PERM: [u8; 3] = [0x05, 0x01, 0x02];

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

    pub fn send_req(&self, prefix: &[u8], raw_data: &[u8]) {
        let plen = prefix.len();
        let len = raw_data.len() + plen;
        let mut ofs = 0;

        let cmd =
            if plen > 0 {
                prefix[0]
            } else {
                raw_data[0]
            };

        while ofs < len {
            let mut data = [0u8; 65];

            data[0] = 0x00;

            data[1] = cmd;

            data[2] = len as u8;
            data[3] = (len >> 8) as u8;
            data[4] = (len >> 16) as u8;

            data[5] = ofs as u8;
            data[6] = (ofs >> 8) as u8;
            data[7] = (ofs >> 16) as u8;

            for i in ofs..std::cmp::min(ofs + 57, len) {
                data[i - ofs + 8] =
                    if i >= plen {
                        raw_data[i - plen]
                    } else {
                        prefix[i]
                    };
            }

            self.dev.write(&data).unwrap();

            ofs += 57;
        }

        if cmd == 0x05 {
            /* Save changes? */
            self.send_req(&[0x13], &[]);
            /* Show profile 1 */
            self.send_req(&[0x04], &[0x01]);
        }
    }

    fn color_cmd_prefix(&self) -> &[u8] {
        if self.make_changes_permanent {
            &Self::COLOR_PERM
        } else {
            &Self::COLOR_TEMP
        }
    }

    pub fn all_keys_raw(&self, raw_keys: &[u8]) {
        let prefix: &[u8] =
            if self.make_changes_permanent {
                &[0x05, 0x01, 0x02, 0x03]
            } else {
                &[0x0f, 0x03]
            };

        self.send_req(&prefix, raw_keys);
    }

    pub fn all_keys(&self, keys: &KeyMap) {
        self.all_keys_raw(keys.raw());
    }

    pub fn pulse(&self, cp: &ColorParam, speed: u8) {
        let rgb = cp.rgb();

        self.send_req(self.color_cmd_prefix(),
                      &[0x06,
                        cp.mode(),
                        rgb.0, rgb.1, rgb.2,
                        speed]);
    }

    pub fn wave(&self, cp: &ColorParam, speed: u8, direction: Direction) {
        let rgb = cp.rgb();

        self.send_req(self.color_cmd_prefix(),
                      &[0x07,
                        cp.mode(),
                        rgb.0, rgb.1, rgb.2,
                        speed,
                        direction as u8]);
    }

    pub fn reactive(&self, cp: &ColorParam, speed: u8, keyup: bool) {
        let rgb = cp.rgb();

        self.send_req(self.color_cmd_prefix(),
                      &[0x09,
                        cp.mode(),
                        rgb.0, rgb.1, rgb.2,
                        speed,
                        !keyup as u8]);
    }

    pub fn reactive_ripple(&self, cp: &ColorParam, speed: u8, keyup: bool) {
        let rgb = cp.rgb();

        self.send_req(self.color_cmd_prefix(),
                      &[0x0a,
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

        self.send_req(self.color_cmd_prefix(),
                      &[0x0b,
                        mode,
                        rgb.0, rgb.1, rgb.2,
                        speed,
                        direction as u8]);
    }

    pub fn gradient(&self, cp: &ColorParam) {
        let mut req = [0u8; 42];

        req[0] = 0x0c;
        cp.gradient().serialize(&mut req[1..42]);
        self.send_req(self.color_cmd_prefix(), &req);
    }

    pub fn fade(&self, cp: &ColorParam, speed: u8) {
        let mut req = [0u8; 44];

        req[0] = 0x0d;
        req[1] = cp.mode();
        cp.gradient().serialize(&mut req[2..43]);
        req[43] = speed;

        self.send_req(self.color_cmd_prefix(), &req);
    }
}

