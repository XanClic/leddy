// SPDX-FileCopyrightText: 2024 Hanna Czenczek <hanna@xanclic.moe>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;

use crate::check_superfluous_params;
use crate::keyboard::Keyboard;


pub fn x_key_ids(kbd: &Keyboard, params: HashMap<&str, &str>)
    -> Result<(), String>
{
    check_superfluous_params(params)?;

    let mut keys = [0u8; 32 * 6 * 3];

    let colors = [
        0xff, 0x00, 0x00,
        0x00, 0xff, 0x00,
        0x00, 0x00, 0xff,
        0xff, 0xff, 0x00,
        0x00, 0xff, 0xff,
        0xff, 0x00, 0xff
    ];

    for i in 0..(32 * 6 * 3) {
        let color_i = ((i / 18) * 3) % 18;
        keys[i] = colors[color_i + i % 3];
    }

    for _ in 0..5 {
        kbd.all_keys_raw(&keys);
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    for i in 0..(32 * 6 * 3) {
        keys[i] = colors[i % 18];
    }

    for _ in 0..5 {
        kbd.all_keys_raw(&keys);
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    Ok(())
}
