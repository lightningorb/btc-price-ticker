# Orb BTC price tracker (beta)

![Orb Price Ticker](https://lnorb.s3.us-east-2.amazonaws.com/images/orb-price-ticker.png)

Want to watch the BTC bull run whilst you work?

This simple rust application for OSX streams the`BTC/USDT` price from Binance, and displays it in the OSX menu bar. It features a few options:

- portfolio mode
- rounding / displaying with decimals
- averaging the last N seconds

## Performance

This btc price ticker was built for high frequency (live / realtime) updates, in a way which would minimally impact the system. Currently the price ticker updates up to once every 100ms. You can change this value (at the top of the makefile) by running `make uninstall`, updating `PLUGIN_TARGET` and doing `make install`.

The memory footprint of the server is quite low, about `4 MiBs`. CPU usage of the server is minimal on the test hardware, less than `1%` (Mac M2 pro max). Swiftbar on the other hand seems higher, about `3%` CPU usage on average.

It is composed of a client, which swiftbar runs every 100ms, and a long-running server that streams price data from Binance, and makes it available via a unix socket.

The Binance websocket reconnects after disconnections. Data usage is approximately 250MiB per day (assuming 24h, so likely less than half of that).

# Beta notice

After some time, there seems to be quite a few client process running simultanously. Once can only assume Swiftbar doesn't stop processes as expected. Please check for updates for once this issue is resolved.

## Cloning

```
$ git clone https://github.com/lightingorb/orb-price-ticker.git && cd orb-price-ticker/
```

## Setting up Swiftbar and Rust in OSX

```
$ brew install swiftbar
```

You'll need to build this application using cargo, here are some brief commands to install cargo in OSX. If these don't work for you, please research how to set up cargo & rust for OSX.

```
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
$ export PATH="$HOME/.cargo/bin:$PATH"
$ rustc --version
$ cargo --version
```

## Building

```
$ make
```

## Installing

```
$ make install
```