.PHONY: all

PLUGIN_TARGET := btc.100ms.bin

build: build_server build_client

all: stop build install start

build_client:
	cargo build --release --bin client

install_client: build_client
	mkdir -p ~/bin/orb_price_ticker
	cp target/release/client ~/bin/orb_price_ticker/orb_price_ticker_client
	ln -sf ~/bin/orb_price_ticker/orb_price_ticker_client ~/Library/Application\ Support/SwiftBar/Plugins/${PLUGIN_TARGET}

build_server:
	cargo build --release --bin server

install_server: build_server
	mkdir -p ~/bin/orb_price_ticker
	cp target/release/server ~/bin/orb_price_ticker/orb_price_ticker_server

install_plist:
	sed "s|HOME_DIR|${HOME}|g" etc/launchers/osx/com.user.orb-price-ticker.plist > ~/Library/LaunchAgents/com.user.orb-price-ticker.plist

install_config:
	if [ ! -f config.toml ]; then cp config.example.toml config.toml; fi
	mkdir -p ~/.config/orb_price_ticker/
	cp config.toml ~/.config/orb_price_ticker/

install: stop install_client install_server install_config install_plist start

start:
	launchctl load ~/Library/LaunchAgents/com.user.orb-price-ticker.plist

stop:
	launchctl unload ~/Library/LaunchAgents/com.user.orb-price-ticker.plist

restart: stop start
	echo "restarting"

clean: stop
	cargo clean

uninstall: stop
	rm -f ~/Library/LaunchAgents/com.user.orb-price-ticker.plist
	rm -rf ~/.config/orb_price_ticker
	rm -rf ~/bin/orb_price_ticker
	rm -f ~/Library/LaunchAgents/com.user.orb-price-ticker.plist
	rm -f ~/Library/Application\ Support/SwiftBar/Plugins/${PLUGIN_TARGET}