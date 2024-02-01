# RP2040 Gpio Expander

Currently a very early work in progress. 

For another project, I needed a GPIO expander for my ESP32 board. I then also required some extra functionality which
isn't really possible with typical GPIO expanders (e.g. PWM outputs for a pulsing LED, a digital output that reliably pulls to ground for 
100ms after a delay of ~50ms every time on startup, etc). After designing and building a board with 2x PF8574s and a few 555 timer circuits to 
achieve this, a few things didn't work as intended in the final board. The pulsating LED circuit didn't work as intended, and the digital output 
on startup was also a bit unreliable (it needs to trigger 100% of the time but was triggering maybe 90% of the time). Making adjustments
to either of these components would require designing an entirely new board each time, so I decided to condense the functionality of all 
these components into a single RP2040 board that runs as an I2C slave. 

Luckily I have a couple of spare RP Picos lying around. This program will initially just run on one of them, but I'll eventually 
design a new board with an RP2040 chip and all the required connectors.

Because this will end up being quite specific to my current project's requirements, I'd recommend anyone with similar requirements 
to fork this repo and modify it to suit their needs. I may write a more generic version of this in the future to use in my own projects 
though. 