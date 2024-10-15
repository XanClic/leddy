// SPDX-FileCopyrightText: 2024 Hanna Czenczek <hanna@xanclic.moe>
// SPDX-License-Identifier: GPL-3.0-or-later

use hidapi::{HidApi, HidDevice};
use crate::types::{ColorParam, Direction, KeyMap};


pub struct Keyboard {
    dev: HidDevice,
    color_cmd_prefix: Vec<u8>,
    profile: u8,

    pub mini: bool,

    pub width: usize,
    pub led_count: usize,

    /* width * height, row first (unmapped entries are 0xff) */
    /* FIXME: Use floating-point coordinates for better precision */
    pub ledmap: Vec<u8>,
}


#[allow(unused)]
impl Keyboard {
    /* LED indices */
    pub const FN_LOCK:          usize = 0;
    pub const ESCAPE:           usize = 1;
    pub const BACKTICK:         usize = 2;
    pub const TAB:              usize = 3;
    pub const CAPS_LOCK:        usize = 4;
    pub const LSHIFT:           usize = 5;
    pub const LCONTROL:         usize = 6;
    pub const F1:               usize = 7;
    pub const TW_1:             usize = 8;
    pub const Q:                usize = 9;
    pub const A:                usize = 10;
    pub const ISO_PIPE:         usize = 11;
    pub const META:             usize = 12;
    pub const F2:               usize = 13;
    pub const TW_2:             usize = 14;
    pub const W:                usize = 15;
    pub const S:                usize = 16;
    pub const Z:                usize = 17;
    pub const LALT:             usize = 18;
    pub const F3:               usize = 19;
    pub const TW_3:             usize = 20;
    pub const E:                usize = 21;
    pub const D:                usize = 22;
    pub const X:                usize = 23;

    pub const F4:               usize = 25;
    pub const TW_4:             usize = 26;
    pub const R:                usize = 27;
    pub const F:                usize = 28;
    pub const C:                usize = 29;

    pub const F5:               usize = 31;
    pub const TW_5:             usize = 32;
    pub const T:                usize = 33;
    pub const G:                usize = 34;
    pub const V:                usize = 35;
    pub const SPACE:            usize = 36;
    pub const F6:               usize = 37;
    pub const TW_6:             usize = 38;
    pub const Y:                usize = 39;
    pub const H:                usize = 40;
    pub const B:                usize = 41;

    pub const F7:               usize = 43;
    pub const TW_7:             usize = 44;
    pub const U:                usize = 45;
    pub const J:                usize = 46;
    pub const N:                usize = 47;

    pub const F8:               usize = 49;
    pub const TW_8:             usize = 50;
    pub const I:                usize = 51;
    pub const K:                usize = 52;
    pub const M:                usize = 53;

    pub const F9:               usize = 55;
    pub const TW_9:             usize = 56;
    pub const O:                usize = 57;
    pub const L:                usize = 58;
    pub const COMMA:            usize = 59;
    pub const RALT:             usize = 60;
    pub const TW_0:             usize = 61;
    pub const MINUS:            usize = 62;
    pub const P:                usize = 63;
    pub const SEMICOLON:        usize = 64;
    pub const DOT:              usize = 65;
    pub const SLASH:            usize = 66;
    pub const F10:              usize = 67;
    pub const EQUAL:            usize = 68;
    pub const LBRACKET:         usize = 69;
    pub const QUOTE:            usize = 70;

    pub const FN:               usize = 72;
    pub const F11:              usize = 73;

    pub const RBRACKET:         usize = 75;
    pub const ISO_BACKSLASH:    usize = 76;
    pub const RSHIFT:           usize = 77;
    pub const MENU:             usize = 78;
    pub const F12:              usize = 79;
    pub const BACKSPACE:        usize = 80;
    pub const ANSI_BACKSLASH:   usize = 81;
    pub const ENTER:            usize = 82;
    pub const RCONTROL:         usize = 83;
    pub const LEFT:             usize = 84;
    pub const DOWN:             usize = 85;
    pub const RIGHT:            usize = 86;
    pub const UP:               usize = 87;
    pub const DELETE:           usize = 88;
    pub const INSERT:           usize = 89;
    pub const PRINT:            usize = 90;
    pub const MUTE_MIC:         usize = 91;
    pub const MUTE_SPEAKER:     usize = 92;
    pub const SCROLL_LOCK:      usize = 93;
    pub const HOME:             usize = 94;
    pub const END:              usize = 95;
    pub const PAGE_DOWN:        usize = 96;
    pub const GAMING_MODE:      usize = 97;
    pub const PAUSE:            usize = 98;
    pub const PAGE_UP:          usize = 99;

    pub const MINI_SIG_PLATE:   usize = 103;

    pub const NUM_LOCK:         usize = 100;
    pub const NUM_7:            usize = 101;
    pub const NUM_4:            usize = 102;
    pub const NUM_1:            usize = 103;
    pub const NUM_0:            usize = 104;
    pub const NUM_2:            usize = 105;
    pub const NUM_5:            usize = 106;
    pub const NUM_8:            usize = 107;
    pub const NUM_SLASH:        usize = 108;
    pub const NUM_ASTERISK:     usize = 109;
    pub const NUM_9:            usize = 110;
    pub const NUM_6:            usize = 111;
    pub const NUM_3:            usize = 112;
    pub const NUM_DECIMAL:      usize = 113;
    pub const NUM_ENTER:        usize = 114;
    pub const NUM_PLUS:         usize = 115;
    pub const NUM_MINUS:        usize = 116;

    pub const VOLUME_KNOB:      usize = 118;

    pub const FULL_SIG_PLATE:   usize = 120;


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

        let mini = dev_info.product_id() == 0x0102;

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
                dev,
                color_cmd_prefix: vec![0x05, 0x01, 0x02],
                profile: 1,

                mini,

                width: if mini { 18 } else { 22 },
                led_count: if mini { 106 } else { 124 },

                ledmap:
                    if mini {
                        vec![
                           1,    0,    7,   13,   19,   25,   31,   37,   43,   49,  103,   55,   67,   73,   79,   90,   93,   98,
                           2,    8,   14,   20,   26,   32,   38,   44,   50,   56,   61,   62,   68,   74,   80,   89,   94,   99,
                           3,    9,   15,   21,   27,   33,   39,   45,   51,   57,   63,   69,   75, 0xff,   81,   88,   95,   96,
                           4, 0xff,   10,   16,   22,   28,   34,   40,   46,   52,   58,   64,   70,   76,   82, 0xff, 0xff, 0xff,
                           5,   11,   17,   23,   29,   35,   41,   47,   53,   59,   65,   66,   71,   77, 0xff, 0xff,   87, 0xff,
                           6,   12, 0xff,   18,   24,   30,   36,   42,   48,   54,   60,   72, 0xff,   78,   83,   84,   85,   86,
                        ]
                    } else {
                        vec![
                           1,    0,    7,   13,   19,   25,   31,   37,   43,   49,  120,   55,   67,   73,   79,   90,   93,   98,   91,   97,   92,  118,
                           2,    8,   14,   20,   26,   32,   38,   44,   50,   56,   61,   62,   68,   74,   80,   89,   94,   99,  100,  108,  109,  116,
                           3,    9,   15,   21,   27,   33,   39,   45,   51,   57,   63,   69,   75, 0xff,   81,   88,   95,   96,  101,  107,  110,  115,
                           4, 0xff,   10,   16,   22,   28,   34,   40,   46,   52,   58,   64,   70,   76,   82, 0xff, 0xff, 0xff,  102,  106,  111, 0xff,
                           5,   11,   17,   23,   29,   35,   41,   47,   53,   59,   65,   66,   71,   77, 0xff, 0xff,   87, 0xff,  103,  105,  112,  114,
                           6,   12, 0xff,   18,   24,   30,   36,   42,   48,   54,   60,   72, 0xff,   78,   83,   84,   85,   86,  104, 0xff,  113, 0xff,
                        ]
                    },
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

