# dtm2txt
A utility to convert dtm files to txt and back. Written in Rust.

## Download
Download the latest version from the
[releases page](https://github.com/Isaac-Lozano/dtm2txt/releases).

## How to use
To use dtm2txt, drag a dtm file or dtm2txt-generated txt file onto the dtm2txt
executable. dtm2txt will then parse, convert, and write a new file with a
different extension.

To view version number, run the executable on its own in a command line.

## txt format
At the beginning of the txt file, there will be a JSON object with dtm
metadata. Most of the time, this metadata won't have to be changed (including
"input_count" as the program will take the number of inputs from the number
of input lines).

After the JSON object is a series of input lines. Each input line contains the
state of each button and axis along with a couple additional inputs. To notate
a pressed button, the letter is written in upper case. To notate an unpressed
button, the letter is written in lowercase. The buttons must be in the order
Start, A, B, X, Y, Z, Up, Down, Left, Right, Left Trigger, Right Trigger.
After the buttons are axises. Every axis is an unsigned number from 0 to 255
(inclusive). This means that for sticks, neutral is 128. Axises must be in the
order Left Pressure, Right Pressure, Analog X, Analog Y, C X, C Y. Following
axis values are optional extra inputs. These inputs may be any of Change Disc,
Reset, Controller Connected, Reserved. Notated as CD, RST, CC, RSV
respectively. These extra inputs only appear if they are input and don't
appear if they aren't used.

An example input line follows.

```
s A b x y z u d l r lt RT   0 255   0 128 128 128 RST
```

In this input line, A and Right Trigger are pressed. Right Pressure is set to
its maximum and all the analog stick is being pushed left. Additionally, the
console is being reset.

## Limitations
This program does not support multiple controllers or Wii remote data,
currently. Multiple controller support might be added at a whim or if interest
is shown, but Wii remote data will likely not be added in.