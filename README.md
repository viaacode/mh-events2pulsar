# mh-events2pulsar

## Synopsis

Webhook callback that receives premis events from MediaHaven and stores them in Apache Pulsar.
The premis events are split up per type, meaning that every type corresponds with a Pulsar topic.
So a Pulsar topic contains all the events of the same event type.

## Prerequisites

* Git
* Rust toolchain: see https://www.rust-lang.org/tools/install.
* Cargo (should be installed along with the Rust toolchain)
* Docker (optional)

Test the installation with rustc --version. (If you get a command not found, either restart the terminal and/or add the ~/.cargo/bin directory to your PATH.)

## Usage
1. Clone this repository:
    `$ git clone https://github.com/viaacode/mh-events2pulsar`

2. Change into the new directory:
    `$ cd mh-events2pulsar`

3. Fill in an .env file (see .env.example for the used environment variables).

## Running locally
1. Export the environment variables:
    `$ export $(cat .env | xargs)`

2. Run the tests:
    `$ cargo test`

3. Run with cargo run:
    `$ cargo run`

## Running using Docker
1. Build the container:
    `$ docker build -t mh-events2pulsar:latest .`

2. Run the container (with specified .env file):
    `$ docker run --env-file .env --rm mh-events2pulsar:latest`