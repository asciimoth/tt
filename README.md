# Terminal Tetris
![screenshot](https://i.imgur.com/VnZX2RC.png)  
Terminal implementation of the most popular game in the world - Tetris.  
Now with simultaneous play on multiple surfaces.  
Written in rust!

## Instalation
### From source
```
$ git clone https://github.com/DomesticMoth/tt.git
$ cd tt
$ cargo build --release --offline
$ cp ./target/release/tt /usr/bin/tt
```

## Usage
```
tt 1.1.0
DomesticMoth
Terminal Tetris
Feature: playing on multiple tables at the same time

USAGE:
    tt [OPTIONS]

OPTIONS:
    -d, --delay <DELAY>      Render delay [default: 10]
    -f, --fields <FIELDS>    Count of feelds [default: 4]
    -h, --height <HEIGHT>    Field height [default: 10]
        --help               Print help information
    -s, --seed <SEED>        PRNG seed [default: 12345]
    -V, --version            Print version information
    -w, --width <WIDTH>      Field width [default: 10]
```
