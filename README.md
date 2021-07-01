# rust-timy

timy is a simple CLI-based alarm clock, which I build along as I learn the basics of Rust.

## Usage
```
# Sets alarm to 5 minutes
timy -m 5

# Sets alarm to 1,5 minutes and half of the volume
timy -m 1 -s 30 -v 0.5
```

In order to be notified with sound, set
```
export TIMY_SOUND_DIR=<path to the music directory>
```

## Parameters
| Short | Long      | Description                         | Values     |
|-------|-----------|-------------------------------------|------------|
| -s    | --seconds | Sets the seconds part of the clock  | [0, 60]    |
| -m    | --minutes | Sets the minutes part of the clock  | [0, 60]    |
| -v    | --volume  | Sets the volume of the sound played | [0.0, 1.0] |
