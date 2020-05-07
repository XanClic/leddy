use hidapi::{HidApi, HidDevice};
use crate::types::{ColorParam, Direction, KeyMap};


pub struct Keyboard {
    dev: HidDevice,
    color_cmd_prefix: Vec<u8>,
    profile: u8,
}


impl Keyboard {
    pub fn new() -> Result<Self, String> {
        let hidapi = HidApi::new().unwrap();

        /* TODO: Look for all matching devices and let the user choose */
        let dev_info =
            match hidapi.device_list().find(|dev|
                dev.vendor_id() == 0x2f0e &&
                (dev.product_id() == 0x0101 || dev.product_id() == 0x0102) &&
                dev.interface_number() == 1)
            {
                Some(di) => di,
                None =>
                    return Err(String::from("No miniSTREAK or STREAK keyboard \
                                             found")),
            };

        if dev_info.product_id() == 0x0101 {
            eprintln!("Warning: leddy may not work properly for (normal-sized) \
                       STREAK keyboards");
        }

        let dev = match dev_info.open_device(&hidapi) {
            Ok(x) => x,
            Err(e) =>
                return Err(format!("Failed to open HID device: {}\n\
                                    Check whether you have the required access \
                                    rights.",
                                   e)),
        };

        Ok(
            Keyboard {
                dev: dev,
                color_cmd_prefix: vec![0x05, 0x01, 0x02],
                profile: 1,
            }
        )
    }

    pub fn software_effect_start(&mut self) {
        self.color_cmd_prefix = vec![0x0f];
    }

    pub fn software_effect_end(&mut self) {
        self.color_cmd_prefix = vec![0x05, self.profile, 0x02];
        self.refresh_profile();
    }

    pub fn set_profile(&mut self, profile: u8) {
        self.profile = profile;
        self.software_effect_end();
    }

    pub fn refresh_profile(&self) {
        self.send_req(&[0x04], &[self.profile]);
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
            self.send_req(&[0x04], &[self.profile]);
        }
    }

    pub fn all_keys_raw(&self, raw_keys: &[u8]) {
        if self.color_cmd_prefix[0] == 0x05 {
            self.send_req(&[0x05, self.profile, 0x02, 0x03], raw_keys);
        } else {
            self.send_req(&[0x0f, 0x03], raw_keys);
        }
    }

    pub fn all_keys(&self, keys: &KeyMap) {
        self.all_keys_raw(keys.raw());
    }

    pub fn pulse(&self, cp: ColorParam, speed: u8) {
        let rgb = cp.rgb();

        self.send_req(self.color_cmd_prefix.as_slice(),
                      &[0x06,
                        cp.mode(),
                        rgb.0, rgb.1, rgb.2,
                        speed]);
    }

    pub fn wave(&self, cp: ColorParam, speed: u8, direction: Direction) {
        let rgb = cp.rgb();

        self.send_req(self.color_cmd_prefix.as_slice(),
                      &[0x07,
                        cp.mode(),
                        rgb.0, rgb.1, rgb.2,
                        speed,
                        direction as u8]);
    }

    pub fn reactive(&self, cp: ColorParam, speed: u8, keyup: bool) {
        let rgb = cp.rgb();

        self.send_req(self.color_cmd_prefix.as_slice(),
                      &[0x09,
                        cp.mode(),
                        rgb.0, rgb.1, rgb.2,
                        speed,
                        !keyup as u8]);
    }

    pub fn reactive_ripple(&self, cp: ColorParam, speed: u8, keyup: bool) {
        let rgb = cp.rgb();

        self.send_req(self.color_cmd_prefix.as_slice(),
                      &[0x0a,
                        cp.mode(),
                        rgb.0, rgb.1, rgb.2,
                        speed,
                        !keyup as u8]);
    }

    pub fn rain(&self, cp: ColorParam, speed: u8, direction: Direction) {
        let rgb = cp.rgb();

        /* This effect does not support rainbow mode */
        let mode =
            match cp {
                ColorParam::Rainbow => (ColorParam::Randomized).mode(),
                _ => cp.mode()
            };

        self.send_req(self.color_cmd_prefix.as_slice(),
                      &[0x0b,
                        mode,
                        rgb.0, rgb.1, rgb.2,
                        speed,
                        direction as u8]);
    }

    pub fn gradient(&self, cp: ColorParam) {
        let mut req = [0u8; 42];

        req[0] = 0x0c;
        cp.gradient().serialize(&mut req[1..42]);
        self.send_req(self.color_cmd_prefix.as_slice(), &req);
    }

    pub fn fade(&self, cp: ColorParam, speed: u8) {
        let mut req = [0u8; 44];

        req[0] = 0x0d;
        req[1] = cp.mode();
        cp.gradient().serialize(&mut req[2..43]);
        req[43] = speed;

        self.send_req(self.color_cmd_prefix.as_slice(), &req);
    }
}

