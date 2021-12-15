# gpi-case-button-remapper
A simple program that remaps button presses from the Retroflag GPi Case.
The program listens for button press events on /dev/input/event0 and sends keyboard events through a virtual USB keyboard.

To build a binary compatible with the raspberry pi zero 2 (armv7), run `make build`.
