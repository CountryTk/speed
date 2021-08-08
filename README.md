# A simple tool to monitor how fast a file is being downloaded

Mainly used it to monitor how fast a file was getting uploaded to my server with netcat

# Usage

1) Build it `cargo b --release`
2) The executable is located in `target/release/`
3) Move that to a desired location and do `./speed -f <FILE_NAME> -s <KNOWN_SIZE>`


# Documentation

-f --file = specifies the file name to track
-s --size = specifies the file size (if known, not mandatory)


