leddy
=====

Linux LED controller for the fnatic miniSTREAK.

Usage
-----

See `leddy --help` for a description of the supported effects.

You may need to run leddy with root rights to change your keyboard’s lighting.

Examples
--------

Some examples for effects:
* `leddy color=rgb:ff4000`: Colors the whole keyboard orange
* `leddy reactive-ripple/keydown`: Creates a ripple (in a random color) whenever
  a key is released
* `leddy wave/color=rainbow`: Lets a rainbow roll over the keyboard
* `leddy rain/direction=down/speed=20/color=rgb:40ff00`: Lets bright green rain
  drops flow slowly over your keyboard
* `leddy gradient/color=gradient:ff0000@0,00ff00@70,0000ff@100`: Creates kind of
  a rainbow gradient from left to right, where green is right of center
* `leddy fade/color=gradient:ff8080@0,3080ff@50,ff8080@100`: Fades between pink
  and blue (note that the color for positions 0 and 100 is the same)

udev rule
---------

You may put your desired effect into a configuration file like
`/etc/leddy.conf`, e.g.:

```
gradient/color=gradient:ff8080@0,ff8080@19,3080ff@20,3080ff@39,d0fff0@40,d0fff0@60,3080ff@61,3080ff@80,ff8080@81,ff8080@100
```

Then you can let `xargs` pass its content to leddy in a udev rule, like so:

```
ACTION=="add", SUBSYSTEM=="usb", ATTRS{idVendor}=="2f0e", ATTRS{idProduct}=="0102" RUN+="/bin/sh -c '/usr/bin/xargs /usr/bin/leddy < /etc/leddy.conf'"
```

Store this as a file in `/etc/udev/rules.d`, and your customization should be
applied on system startup or whenever the keyboard is plugged in.
