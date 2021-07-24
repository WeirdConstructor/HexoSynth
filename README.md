# HexoSynth - A hexagonal modular synthesizer

This project aims to create a modular synthesizer. Like those encountered
in projects like VCVRack or Bitwigs Polygrid.

The core idea is having a hexagonal tile map for laying out module
instances and connect them at the edges to route audio signals and CV signals
to inputs of other modules.

A goal is to provide a simple wireless environment to build
sound effects, synthesizers or whole generative music patches from
predefined modules.

Hosting plugins (VST, LV2, ...) is out of the scope of this project.
The goal is rather to have a good set of predefined modules.

## State of Development

This project is still (2021-06-22) under heavy development and is considered
pre alpha stage. There are only few nodes (aka modules) implemented yet. The
interface is mostly functional though. But not tested in all hosts yet and
there are known bugs.

Make sure to follow [Weird Constructors Mastodon
account](https://mastodon.online/@weirdconstructor) or the releases of this
project to be notified once I release a beta or stable release.

### Road Map / TODO List

I have a pretty detailed TODO list in my private notebook, but
this is the rough road map:

- DONE: Make a UI that is more or less fluently usable and easily extendable
with new modules.
- DONE: Take a bit of care that there is online help.
- DONE: Factor out the DSP code into it's own crate.
- Add lots of modules (Oscillators, Filters, Envelopes, LFOs, Quantizers, ...).
- Add a MIDI-CV interface for receiving pitch CV, gate and clock from the DAW
- Add parameter input node for receiving automation from the DAW
- Add audio inputs for receiving audio from the DAW
- Comment the code for easier maintenance.

## Building and Dependencies

You need nightly rust:

    rustup toolchain install nightly


You might need following dependencies (Ubuntu Linux):

    sudo apt install libjack0 libjack-jackd2-dev qjackctl libx11-xcb-dev
    sudo apt install libxcb-icccm4-dev libxcb-dri3-dev

You might need following dependencies (Ubuntu 20.04 Linux):

    sudo apt install libgl1-mesa-dev libjack-jackd2-dev qjackctl libxcursor-dev
    sudo apt install libx11-xcb-dev libxcb-icccm4-dev libxcb-dri2-0-dev libxcb-dri3-dev

These might work on Debian too:

    sudo apt install libjack0 libjack-dev libx11-xcb-dev libxcb-icccm4-dev libxcb-dri2-dev

### Compiling the VST plugin

Enter the `vst2` subdirectory:

    hexosynth/$ cd vst2

Compile:

    hexosynth/vst2/$ cargo +nightly build --release

Install:

    heyosynth/vst2/$ cp target/release/libhexosynth_vst.so ~/.vst/

### Running the Standalone Example

Enter the `jack_standalone` subdirectory:

    hexosynth/$ cd jack_standalone

Compile and run:

    hexosynth/jack_standlone/$ cargo +nightly run --release --example standalone

## Running the Automated Testsuite:

There exists an automate test suite for the DSP and backend code:

    cargo test --release -- --nocapture

## DAW Compatibility

As of 2021-06-22 HexoSynth has been tested with:

    - Windows 10 and Ableton Live: It starts, and you can use it via the mouse.
      The keyboard handling is not working properly though.
    - Ubuntu Linux 20.04 and Carla: Works
    - Ubuntu Linux 20.04 and Bitwig: Works

## Known Bugs

* The ones you encounter and create as issues on GitHub.

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
of this project (AGPLv3 or later).

## Contact the Author

You can reach me via Discord or Mastodon. I'm joined most public Rust Discord
servers, especially the "Rust Audio" Discord server. I am also on freenode.net,
for instance in the `#lad` channel (nick `weirdctr`).

## Support Development

You can support me (and the development of this project) via Liberapay:

<a href="https://liberapay.com/WeirdConstructor/donate"><img alt="Donate using Liberapay" src="https://liberapay.com/assets/widgets/donate.svg"></a>

## License

This project is licensed under the GNU Affero General Public License Version 3 or
later.

The fonts DejaVuSerif.ttf and DejaVuSansMono.ttf under the license:

    Fonts are (c) Bitstream (see below). DejaVu changes are in public domain.
    Glyphs imported from Arev fonts are (c) Tavmjong Bah (see below)

### Why (A)GPL?

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

Most of those reasons don't yet justify (A)GPL. The main point of the (A)GPL, as far
as I understand: The (A)GPL makes sure the software stays free software until
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

This makes two reasons for me to choose the (A)GPL:

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
