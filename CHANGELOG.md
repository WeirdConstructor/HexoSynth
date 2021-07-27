0.1.0-alpha-3 (unreleased)
==========================

* Feature: New node added: 'Mix3' a simple 3 channel mixer node
to sum 3 signals.
* Feature: New node added: 'BOsc' a basic oscillator
for sine, triangle, saw and pulse waveforms.
* Feature: TSeq module documentation shows a value cheat sheet,
to quickly compose gates and values in your sequences.
* Feature: The min/max signal monitors also print the min/max/average values
of the signal that is visible in the monitor.
* Change: The min/max signal monitors are now wider and display 3 seconds
of the signal instead of only 2.
* Change: The patch file format now stores input/output port names
now instead of indices. Current format with port indices is still
loaded correctly.
* Change: 'Sin' node now has a randomized initial phase, except for the
very first instance 'Sin(0)'.
* Change: The signal scopes draw the center line no longer above the
waveform.
* Bugfix: Note columns in the tracker did not show the note name.
* Project: GUI test suite can now place matrix cells directly.
* Project: Moved GUI tests suite to it's own sub directory 'gui\_tests'
as separate application.

0.1.0-alpha-2 (2021-07-24)
==========================

* Bugfix: Keyboard events should now be properly forwarded from the Host
via the VST2 API. Confirmed to work in Ardour 6.
* Bugfix: Version label is now wider with a smaller font.
* Change: Middle mouse button in fine adj area removes modulation amount.
* Change: Resized the window from 1400x700 to 1400x787 to fit into the
Full HD aspect.
* Project: Added two sub crates: jack\_standalone and vst2


0.1.0-alpha-1 (2021-07-23)
==========================

* Initial pre-release for testing purposes.
