# Used to (re)build the fixture tarball + sha256 for install_sh tests.
all: cchud-darwin-arm64.tar.gz cchud-darwin-arm64.tar.gz.sha256 \
     cchud-linux-x64.tar.gz cchud-linux-x64.tar.gz.sha256 \
     cchud-linux-x64-musl.tar.gz cchud-linux-x64-musl.tar.gz.sha256 \
     cchud-darwin-x64.tar.gz cchud-darwin-x64.tar.gz.sha256

cchud-%.tar.gz: cchud-fixture/cchud
	cd cchud-fixture && tar czf ../$@ cchud

cchud-%.tar.gz.sha256: cchud-%.tar.gz
	(sha256sum $< 2>/dev/null || shasum -a 256 $<) > $@

clean:
	rm -f *.tar.gz *.sha256
