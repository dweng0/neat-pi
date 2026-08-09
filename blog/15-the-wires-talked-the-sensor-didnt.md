---
title: "The wires talked, the sensor didn't"
episode: 15
pubDate: 2026-08-07
sessionDate: 2026-08-07
status: published
teaser: "A connector with no labels gave up its secret to nothing but a multimeter — I knew every wire before I applied a single volt. Then I powered it up, turned the wheel, and it had absolutely nothing to say. And the night ended with a flash and every light going out."
heroPhoto: encoder-board-wired-pinout-1.jpg
seeAlso: [reference/handoff]
---

Last time the wheel spun because I told it to — but it had no idea it had moved. That's the four wires I taped aside: the encoder, the thing that turns a spinning motor into *distance*. Tonight was supposed to be the night the robot learned to count its own steps.

It started with a proper little detective win, and I'll take it.

The encoder board has no pin labels — just `LEGO WHEEL ENCODER ASY 915-1055` silkscreen and six wires: red, black, brown, orange, blue, yellow. Red and black I already knew were the motor. The other four were `Vcc`, `GND`, and two signals in some unknown order, and I was determined not to guess and fry the chip. So, meter first.

My first move went sideways in a useful way. I found a little capacitor next to the sensor and buzzed each wire against it, expecting to catch the two power wires — the theory being a decoupling cap sits across `Vcc` and `GND`. Instead it only beeped to *red and black*. That wasn't the logic cap at all; it was the motor's noise-suppression cap, and the beep was the meter reading straight through the `~6 Ω` motor winding. Wrong cap — but it re-confirmed red and black are the motor, so, fine.

Then the real trick. I put the meter on resistance and measured every pair of the four thin wires. Most read open. But brown-to-blue came back `2.4 kΩ`, and brown-to-yellow `2.43 kΩ` — two nearly identical numbers. That's a fingerprint. It means brown feeds both blue and yellow through matched pull-up resistors, which is exactly what you'd see if **brown is `Vcc`** and blue and yellow are the two open-drain outputs. That left orange as `GND` by elimination — and brown-to-orange reading open told me the power rails weren't shorted, so it was safe to power. I even pulled up the close-up photo afterward and the wires solder to the pads in that exact order: motor+, `Vcc`, `GND`, A, B. Decoded a label-less connector from resistance alone, before touching a power supply. Great feeling.

It lasted about ten minutes.

I wired brown to `3.3 V`, orange to ground, put the meter on the blue output, and turned the wheel by hand. It should idle high and snap toward zero as the magnet poles pass. It sat at `2.93 V`. Rock steady. Turned it slow, turned it fast, inched it round — `2.93`, always. Bumped the supply to `5 V`. Now it sat at `4.52 V`, just as steadily. Every wire, every voltage, every speed: a flat line.

So I did what I should've done sooner and looked up the actual part. It's a **magnetic disc with 8 poles** on the motor shaft, read by a **unipolar Hall switch** — an `A3144`-class chip. Two things clicked. One: those chips need `4.5 V` minimum, so my `3.3 V` test was doomed from the start. Two: my ESP32's `5 V` pin was sagging to `4.52 V` — right on the ragged edge of the spec. So I dragged out the Arduino Uno purely as a clean regulator: 12 V into its barrel jack, and its `5 V` pin held a solid `4.96 V`. Powered the encoder from that. Turned the wheel.

`4.57 V`. Nothing.

Here's the honest, annoying part: I couldn't actually *close the loop* on why. A Hall switch you can test dead simple — wave a magnet at it and watch the output flip. But the only magnet in the house was a fridge magnet, and a fridge magnet is far too weak to trip an `A3144`, so its silence proved precisely nothing. And it dawned on me I'd never once *watched the motor shaft physically turn* during my "spin it under power" tests — so for all I knew, the magnet had never moved at all. Two different unknowns, and I couldn't rule out either without a proper magnet I didn't own.

And then, reaching across the bench to sort the wiring out, the ground wire flopped loose and landed square on the Uno's ICSP header — which carries `5 V` and ground right next to each other. A flash. And every LED on the board went dark.

I shorted the 5 V rail. I unplugged it fast; it might come back on USB, it might not. That's a Monday problem now.

So the wires talked and I understood every one of them — and the sensor still hasn't said a single word back. Monday: a real magnet, raided out of a dead hard drive. Find out if the Uno survived. And finally answer the only question that matters — is that sensor dead, or has it just never once seen a magnet move?
