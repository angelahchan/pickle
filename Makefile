ifndef MAXMIND_KEY
$(error "Please set MAXMIND_KEY.")
endif

OUT = build

.PHONY: run client server

$(OUT)/geolite2-city.mmdb:
	curl -L 'https://download.maxmind.com/app/geoip_download?edition_id=GeoLite2-City&license_key=$(MAXMIND_KEY)&suffix=tar.gz' -o $@
	tar -xf $@ $$(tar -tf $@ | grep -m1 '.mmdb$$') --transform 's!.*!$@!'

server:
	cd server && cargo build --release --out-dir ../$(OUT) -Z unstable-options

client:
	parcel build -d $(OUT)/static client/src/index.html

run: client server $(OUT)/geolite2-city.mmdb
	cd $(OUT) && ./pickle
