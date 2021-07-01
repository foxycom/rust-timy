# rust-timy

timy is a simple CLI-based alarm clock, which I build along as I learn the basics of Rust.

## Usage
```
# Set alarm to 5 minutes
timy -m 5
```

## Parameters
| Short | Long      | Description                         | Values     |
|-------|-----------|-------------------------------------|------------|
| -s    | --seconds | Sets the seconds part of the clock  | [0, 60]    |
| -m    | --minutes | Sets the minutes part of the clock  | [0, 60]    |
| -v    | --volume  | Sets the volume of the sound played | [0.0, 1.0] |
