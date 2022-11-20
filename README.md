# HexoSynth - A hexagonal modular synthesizer

[![Build](https://github.com/WeirdConstructor/HexoSynth/actions/workflows/build.yml/badge.svg)](https://github.com/WeirdConstructor/HexoSynth/actions/workflows/build.yml)

This project aims to create a modular synthesizer plugin (VST3, CLAP). Like
those encountered in projects like VCVRack or Bitwig's Grid.

The core idea is having a hexagonal tile map for laying out module
instances and connect them at the edges to route audio signals and control signals
to inputs of other modules.

A goal is to provide a simple wireless environment to build
sound effects, synthesizers or whole generative music patches from
predefined modules.

Hosting plugins (VST, LV2, ...) is out of the scope of this project.
The goal is rather to have a good set of predefined modules.

Here is a screenshot of how it looks:

![HexoSynth Screenshot from 2022-09-11](https://github.com/WeirdConstructor/HexoSynth/raw/master/screenshots/2022-09-11_08-07.png)

## State of Development

Since June 2022 the project is under heavy development again.  A rewrite of
HexoTK took longer than anticipated, but provides all required features now.
All development currently takes place on the "master" branch, so don't be surprised
if something does not work. As of
2022-08-16 most of the functionality from end of 2021 has been rewritten and
new features and polish are being added right now for the next release.

If you want to stay up to date, follow my devlog:

- https://m8geil.de/posts/hexosynth-1/
- https://m8geil.de/posts/hexosynth-2/
- https://m8geil.de/posts/hexosynth-3/
- https://m8geil.de/posts/hexosynth-4/
- https://m8geil.de/posts/hexosynth-5/
- https://m8geil.de/posts/hexosynth-6/
- https://m8geil.de/posts/hexosynth-7/
- https://m8geil.de/posts/hexosynth-8/
- https://m8geil.de/posts/hexosynth-9/
- https://m8geil.de/posts/hexosynth-10/
- https://m8geil.de/posts/hexosynth-11/
- https://m8geil.de/posts/hexosynth-12/
- for an up to date list, look here: https://m8geil.de/tags/hexosynth/

Make sure to follow [Weird Constructors Mastodon
account](https://mastodon.online/@weirdconstructor) or the releases of this
project to be notified once I release a beta or stable release.

If you want to do chat, feel free to join the RustAudio Discord / Community
here: https://rust.audio/

### Rough Code Structure

HexoSynth leaves all GUI logic to the scripting language
[WLambda](http://wlambda.m8geil.de/), which is an easily embeddable scripting
language for Rust. All higher level functionality will be realised in WLambda.
As well as the test suite for the GUI.

The Rust code contains all the low level functionality, such as the DSP code,
the hexagonal grid data structure and the implementation of all the GUI
widgets.

The scripting language code will be linked into the HexoSynth application at
compile time. The goal is to deploy a single binary. But you could load the
WLambda code from a different place using an environment variable later.

In the process of developing HexoSynth I developed a series of crates (aka libraries)
that factor out some generic parts to be reusable by someone else maybe:

- [HexoDSP - The DSP backend of HexoSynth](https://github.com/WeirdConstructor/HexoDSP)
- [HexoTK - The GUI toolkit of HexoSynth](https://github.com/WeirdConstructor/HexoTK)
- [SynFx-DSP - A collection of DSP functions and tools to support HexoDSP](https://github.com/WeirdConstructor/synfx-dsp)
- [SynFx-DSP-JIT - A DSP JIT (just in time) compiler for the WBlockDSP visual langauge](https://github.com/WeirdConstructor/synfx-dsp-jit)

### Implemented Features

- A useable GUI with highly features parameter knobs and
a hexagonal module/node matrix that is easily changeable with the mouse.
- Serialization/Deserialization of patches is implemented in the VST3/CLAP plugins
and can be managed by the DAW.
- Signal monitors for the selected node.
- Per node and output signal feedback LEDs.
- A simple Tracker to edit "TSeq" sequences.
- A primitive sample selection browser.
- Prototype of the WBlockDSP visual programming language for DIY DSP nodes inside HexoSynth.

And following DSP nodes:

| Category | Name | Function |
|-|-|-|
| IO Util | Out         | Audio output (to DAW or Jack) |
| Osc     | Sampl       | Sample player |
| Osc     | Sin         | Sine oscillator |
| Osc     | BOsc        | Basic bandlimited waveform oscillator (waveforms: Sin, Tri, Saw, Pulse/Square) |
| Osc     | VOsc        | Vector phase shaping oscillator |
| Osc     | Noise       | Noise oscillator |
| Osc     | FormFM      | Formant oscillator based on FM synthesis |
| Signal  | Amp         | Amplifier/Attenuator |
| Signal  | SFilter     | Simple collection of filters, useable for synthesis |
| Signal  | FVaFilt     | Collection of virtual analog filters (Moog, EDP Wasp, Korg MS20) |
| Signal  | Delay       | Single tap signal delay |
| Signal  | PVerb       | Reverb node, based on Dattorros plate reverb algorithm |
| Signal  | AllP        | All-Pass filter based on internal delay line feedback |
| Signal  | Comb        | Comb filter |
| N-\>M   | Mix3        | 3 channel mixer |
| N-\>M   | Mux9        | 9 channel to 1 output multiplexer/switch |
| Ctrl    | SMap        | Simple control signal mapper |
| Ctrl    | Map         | Control signal mapper |
| Ctrl    | CQnt        | Control signal pitch quantizer |
| Ctrl    | Quant       | Pitch signal quantizer |
| Mod     | TSeq        | Tracker/pattern sequencer |
| Mod     | Ad          | Attack-Decay (AD) envelope |
| Mod     | Adsr        | Attack-Decay-Sustain-Release (ADSR) envelope |
| Mod     | TsLFO       | Tri/Saw waveform low frequency oscillator (LFO) |
| Mod     | RndWk       | Random walker, a Sample & Hold noise generator |
| IO Util | FbWr / FbRd | Utility modules for feedback in patches |
| IO Util | Scope       | Oscilloscope for up to 3 channels |
| IO Util | MidiP       | MIDI Pitch/Note input from plugin host, DAW or hardware |
| IO Util | MidiCC      | MIDI CC input from plugin host, DAW or hardware |
| IO Util | ExtA - ExtF | Access to plugin parameter sets A to F |

### Road Map / TODO List

I have a pretty detailed TODO list in my private notebook, but
this is the rough road map:

- DONE: Make a UI that is more or less fluently usable and easily extendable
with new modules.
- DONE: Take a bit of care that there is online help.
- DONE: Factor out the DSP code into it's own crate.
- DONE: Redo the UI with performance optimized and overhauled HexoTK.
- DONE: Rebuild the UI logic of HexoSynth from 2021 with WLambda.
- Add preset/patch management to the UI.
- Add lots (many more than above listed) of modules (Oscillators, Filters, Envelopes, LFOs, Quantizers, ...).
- Add a MIDI-Ctrl interface for receiving pitch control signals, gate and clock from the DAW
- Add parameter input node for receiving automation from the DAW
- Add audio inputs for receiving audio from the DAW
- Comment the code for easier maintenance.

## Building and Dependencies

You might need following dependencies (Ubuntu Linux):

    sudo apt install libjack0 libjack-jackd2-dev qjackctl libx11-xcb-dev
    sudo apt install libxcb-icccm4-dev libxcb-dri3-dev

You might need following dependencies (Ubuntu 20.04 Linux):

    sudo apt install libgl1-mesa-dev libjack-jackd2-dev qjackctl libxcursor-dev
    sudo apt install libx11-xcb-dev libxcb-icccm4-dev libxcb-dri2-0-dev libxcb-dri3-dev

These might work on Debian too:

    sudo apt install libjack0 libjack-dev libx11-xcb-dev libxcb-icccm4-dev libxcb-dri2-dev

### Compiling the VST3 and CLAP plugins

Compile:

    $ cargo +nightly xtask bundle hexosynth_plug --release

Install:

    $ cp -vfr target/bundled/hexosynth_plug.vst3 ~/.vst3/
    $ cp -vfr target/bundled/hexosynth_plug.clap ~/.vst3/

### Running the CPAL Standalone Example

CPAL is a generic audio device abstraction library. It should work
on most systems.

Compile and run:

    $ cargo +nightly run --release --bin hexosynth_cpal

### Running the Jack Standalone Example

JACK Audio Connection Kit is a sound server API, which allows
multiple audio applications to communicate with each other.

Compile and run:

    $ cargo +nightly run --release --bin hexosynth_jack

## DAW Compatibility

As of 2022-08-15 HexoSynth has been tested with:

    - Ubuntu Linux 20.04 and Bitwig: Works
    - Ubuntu Linux 20.04 and Renoise: Works
    - Ubuntu Linux 20.04 and Reaper: Works, except Keyboard support
    - Ubuntu Linux 20.04 and Ardour: Works

## Known Bugs

* The ones you encounter and create as issues on GitHub.

## Credits

- Dimas Leenman (aka Skythedragon) contributed the `FormFM` node.
- Frederik Halkjær (aka Fredemus, aka RocketPhysician) contributed the DSP algorithms
for the `FVaFilt` virtual analog filter node.

## Contributions

I currently have a quite precise vision of what I want to achieve and my goal
is to make music with this project eventually.

The projects is still young, and I currently don't have that much time to
devote for project coordination. So please don't be offended if your issue rots
in the GitHub issue tracker, or your pull requests is left dangling around
for ages.

I might merge pull requests if I find the time and think that the contributions
are in line with my vision.

Please bear in mind, that I can only accept contributions under the License
of this project (GPLv3 or later).

### Help

If you want to help this project:

- **Samples**: Find drum or synth samples that I can publish in HexoSynth. I would love
to provide a drum kit with default sounds that is always available.
- **Binaries**: Build Windows and/or Mac OS binaries: I've decided that I won't build windows binaries
anymore. I don't like supporting the whole Apple & Micosoft vendor lock-in directly.
However, I would be fine putting links to your binaries on my release pages
or the HexoSynth README. Given that the binaries are current enough.
- **More DSP nodes/modules**: Implement more DSP nodes: You could extend [HexoDSP](https://github.com/WeirdConstructor/HexoDSP)
with new DSP implementations. Just reach out to me via Discord/IRC/Mastodon or Github
issue to discus the name of the new node.
I wrote up a little guide to get you started here:
Refer to the [HexoDSP API Documentation - DSP node implementation guide](http://m8geil.de/hexodsp_doc/hexodsp/dsp/index.html).

## Contact the Author

You can reach me via Discord ( WeirdConstructor#7936 ), Mastodon (
@weirdconstructor@mastodon.online ) or IRC. I'm joined most public Rust Discord
servers, especially the "Rust Audio" Discord server. And I am also on IRC on
the network [Libera.Chat](https://libera.chat/) in the `#lad` channel (nick `wct`).

If you don't have means to access any of them, you can alternatively
send me a Github issue.

Don't use E-Mail, I only read them irregularly, and I might miss yours completely.

## Support Development

You can support me (and the development of this project) via Liberapay:

<a href="https://liberapay.com/WeirdConstructor/donate"><img alt="Donate using Liberapay" src="https://liberapay.com/assets/widgets/donate.svg"></a>

## License

This project is licensed under the GNU General Public License Version 3 or
later.

The fonts DejaVuSerif.ttf and DejaVuSansMono.ttf under the license:

    Fonts are (c) Bitstream (see below). DejaVu changes are in public domain.
    Glyphs imported from Arev fonts are (c) Tavmjong Bah (see below)

### Why GPL?

Picking a license for my code bothered me for a long time. I read many
discussions about this topic. Read the license explanations. And discussed
this matter with other developers.

First about _why I write code for free_ at all, the reasons are:

- It's my passion to write computer programs. In my free time I can
write the code I want, when I want and the way I want. I can freely
allocate my time and freely choose the projects I want to work on.
- To help a friend or member of my family.
- To solve a problem I have.
- To learn something new.

Those are the reasons why I write code for free. Now the reasons
_why I publish the code_, when I could as well keep it to myself:

- So that it may bring value to users and the free software community.
- Show my work as an artist.
- To get into contact with other developers.
- To exchange knowledge and help other developers.
- And it's a nice change to put some more polish on my private projects.

Most of those reasons don't yet justify GPL. The main point of the GPL, as far
as I understand: The GPL makes sure the software stays free software until
eternity. That the _end user_ of the software always stays in control. That the users
have the means to adapt the software to new platforms or use cases.
Even if the original authors don't maintain the software anymore.
It ultimately prevents _"vendor lock in"_. I really dislike vendor lock in,
especially as developer. Especially as developer I want and need to stay
in control of the computers and software I use.

Another point is, that my work (and the work of any other developer) has a
value. If I give away my work without _any_ strings attached, I effectively
work for free. This compromises the price I (and potentially other developers)
can demand for the skill, workforce and time.

This makes two reasons for me to choose the GPL:

1. I do not want to support vendor lock in scenarios for free.
   I want to prevent those when I have a choice, when I invest my private
   time to bring value to the end users.
2. I don't want to low ball my own (and other developer's) wage and prices
   by giving away the work I spent my scarce private time on with no strings
   attached. I do not want companies to be able to use it in closed source
   projects to drive a vendor lock in scenario.

We can discuss relicensing of my code or project if you are interested in using
it in a closed source project. Bear in mind, that I can only relicense the
parts of the project I wrote. If the project contains GPL code from other
projects and authors, I can't relicense it.
