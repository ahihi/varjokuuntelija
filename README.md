# Varjokuuntelija

A fragment shader renderer for prototyping/livecoding. [Video](http://omino.foldplop.com/~ahihi/Varjokuuntelija-demo.mp4)

![Screenshot](http://i.imgur.com/GMtEpWS.png)

## Features

* Auto-reload shader file on changes (via [notify](https://github.com/passcod/rsnotify))
* Map MIDI CC input to shader uniforms

## Usage

```
Usage: varjokuuntelija [options] FILE

Options:
    -c, --config FILE   configuration file
    -w, --width PIXELS  resolution width
    -h, --height PIXELS resolution height
    -f, --fullscreen INDEX
                        enable full screen mode on display INDEX
```

### MIDI input

Specify MIDI mappings in a configuration file and use `--config`. The included example configuration (`example-config.json`) maps CC messages 21/22/23 from channel 1 of MIDI device 0 to the uniforms `u_midi_red`/`green`/`blue`. These are used by `example-shader.frag` to render a simple RGB mixer.

```
{
  "midi": {
    "0": {
      "1": {
        "21": "u_midi_red",
        "22": "u_midi_green",
        "23": "u_midi_blue"
      }
    }
  }
}
```
