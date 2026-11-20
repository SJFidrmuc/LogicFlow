# FAQ

## Windows says the app is unsafe. Is it safe?

Windows SmartScreen warns about every downloaded program that does not yet have a digital signature and reputation. LogicFlow is open source, its code is public on GitHub and the binaries are built automatically by GitHub.

1. In the *Windows protected your PC* window click **More info** and then **Run anyway**.
2. To stop the warning from coming back, right-click the downloaded ZIP before extracting → **Properties**, tick **Unblock** and confirm with OK.

At school, the network administrator can allow the program on all computers at once. A signed version is planned.

## The program does not open on a Mac.

The program is not yet signed for macOS. On first start right-click `logicflow` and choose **Open**.

## The program does not start on a Raspberry Pi.

You need a Raspberry Pi 4 or 5 with the 64-bit Raspberry Pi OS. Older models and the 32-bit system are not supported.

## Why is a wire red and the LED showing X?

The value is undefined. Most often a gate input is unconnected or two outputs drive the same wire. Circuits with memory start in state X until you set or reset them.

## I drew a wire but the signal does not pass.

Check that there is a dot at the junction. Wires that merely cross are not connected. A component pin connects only to a wire end, not to a wire passing over it.

## The truth table is empty.

The table needs at least one input (switch or button) and one LED. It is computed for at most 8 inputs.

## The program says “The circuit oscillates”.

The circuit has no stable state, for example an inverter feeding itself or an odd number of inverters in a loop. Check the feedback.

## The clock generator inside a subcircuit does not run.

That is intended. Feed the clock in through a subcircuit input.

## How do I find out my version?

Help → About, or bottom right in the status bar. The latest version is on [www.logicflow-sim.cz](https://www.logicflow-sim.cz).

## Can I use LogicFlow on a tablet or phone?

Not yet. A web version for tablets and Chromebooks is planned after version 1.0.

## May a school use the program for free?

Yes. LogicFlow is released under the MIT licence, so anyone may use, copy and distribute it, at school and at work.

## I found a bug. Where do I report it?

On [GitHub → Issues](https://github.com/SJFidrmuc/LogicFlow/issues). Please state the program version and your system, and attach the `.logicflow` file if the bug concerns a specific circuit.
