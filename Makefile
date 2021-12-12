# These are valid.
YAML_SAFE_OBJECTS = \
	$(wildcard data/relation-*.yaml) \
	data/relations.yaml \

# These are well-formed.
YAML_OBJECTS = \
	$(YAML_SAFE_OBJECTS) \
	.github/workflows/tests.yml \
	data/refcounty-names.yaml \
	data/refsettlement-names.yaml \

YAML_TEST_OBJECTS = \
	$(wildcard tests/data/relation-*.yaml) \
	tests/data/relations.yaml \
	tests/data/refcounty-names.yaml \
	tests/data/refsettlement-names.yaml \

TS_OBJECTS = \
	src/browser/config.ts \
	src/browser/main.ts \
	src/browser/stats.ts \
	src/browser/types.d.ts \

RS_OBJECTS = \
	src/accept_language.rs \
	src/area_files.rs \
	src/areas.rs \
	src/bin/cache_yamls.rs \
	src/bin/cron.rs \
	src/bin/missing_housenumbers.rs \
	src/bin/parse_access_log.rs \
	src/bin/rouille.rs \
	src/bin/validator.rs \
	src/cache.rs \
	src/cache_yamls.rs \
	src/context.rs \
	src/cron.rs \
	src/i18n.rs \
	src/lib.rs \
	src/missing_housenumbers.rs \
	src/overpass_query.rs \
	src/parse_access_log.rs \
	src/ranges.rs \
	src/stats.rs \
	src/util.rs \
	src/validator.rs \
	src/webframe.rs \
	src/wsgi.rs \
	src/wsgi_additional.rs \
	src/wsgi_json.rs \
	src/yattag.rs \

# Source local config if it's there.
-include config.mak

ifdef RSDEBUG
CARGO_OPTIONS =
TARGET_PATH = debug
else
CARGO_OPTIONS = --release
TARGET_PATH = release
endif
CARGO_OPTIONS += --color always

ifndef V
	QUIET_MSGFMT = @echo '   ' MSGMFT $@;
	QUIET_ESLINT = @echo '   ' ESLINT $@;
	QUIET_VALIDATOR = @echo '   ' VALIDATOR $@;
endif

all: builddir/bundle.js css wsgi.ini data/yamls.cache locale/hu/LC_MESSAGES/osm-gimmisn.mo build

clean:
	rm -f config.ts
	rm -f $(patsubst %.yaml,%.validyaml,$(YAML_SAFE_OBJECTS))
	rm -rf $(patsubst %.ts,%.eslint,$(TS_OBJECTS)) builddir

check: all check-filters check-unit check-eslint check-rustfmt check-clippy
	@echo "make check: ok"

check-rustfmt: Cargo.toml $(RS_OBJECTS)
	cargo fmt -- --check && touch $@

check-clippy: Cargo.toml $(RS_OBJECTS)
	cargo clippy ${CARGO_OPTIONS} && touch $@

build: $(RS_OBJECTS) Cargo.toml Makefile
	cargo build ${CARGO_OPTIONS}

# Without coverage: cargo test --lib -- --test-threads=1
check-unit: Cargo.toml $(RS_OBJECTS) locale/hu/LC_MESSAGES/osm-gimmisn.mo testdata data/yamls.cache
	cargo tarpaulin --lib -v --skip-clean --fail-under 100 --target-dir ${PWD}/target-cov ${CARGO_OPTIONS} -- --test-threads=1

src/browser/config.ts: wsgi.ini Makefile
	printf 'const uriPrefix = "%s";\nexport { uriPrefix };\n' $(shell grep prefix wsgi.ini |sed 's/uri_prefix = //') > $@

ifdef TSDEBUG
BROWSERIFY_OPTIONS = --debug
else
BROWSERIFY_OPTIONS = --plugin tinyify
endif
BROWSERIFY_OPTIONS += --plugin tsify

builddir/bundle.js: $(TS_OBJECTS) package-lock.json
	mkdir -p builddir
	node_modules/.bin/browserify -o builddir/bundle.js $(BROWSERIFY_OPTIONS) $(TS_OBJECTS)

package-lock.json: package.json
	npm install
	touch package-lock.json

css: workdir/osm.min.css

workdir/osm.min.css: static/osm.css package-lock.json
	mkdir -p workdir
	[ -x "./node_modules/.bin/cleancss" ] && ./node_modules/.bin/cleancss -o $@ $< || cp -a $< $@

testdata: tests/data/yamls.cache tests/workdir/osm.min.css tests/favicon.ico tests/favicon.svg

tests/favicon.ico: favicon.ico
	cp -a $< $@

tests/favicon.svg: favicon.svg
	cp -a $< $@

tests/workdir/osm.min.css: workdir/osm.min.css
	mkdir -p tests/workdir
	cp -a $< $@

# Intentionally don't update this when the source changes.
wsgi.ini:
	cp data/wsgi.ini.template wsgi.ini

data/yamls.cache: build $(YAML_OBJECTS)
	target/${TARGET_PATH}/cache_yamls data workdir

tests/data/yamls.cache: build $(YAML_TEST_OBJECTS)
	target/${TARGET_PATH}/cache_yamls tests/data tests/workdir

check-eslint: $(patsubst %.ts,%.eslint,$(TS_OBJECTS))

%.eslint : %.ts Makefile .eslintrc
	$(QUIET_ESLINT)node_modules/.bin/eslint $< && touch $@

check-filters: $(patsubst %.yaml,%.validyaml,$(YAML_SAFE_OBJECTS))

%.validyaml : %.yaml build
	$(QUIET_VALIDATOR)target/${TARGET_PATH}/validator $< && touch $@

# Make sure that the current directory is *not* the repo root but something else to catch
# non-absolute paths.
run: all
	cd $(HOME) && $(PWD)/target/${TARGET_PATH}/rouille

deploy:
ifeq (,$(wildcard ./deploy.sh))
	git pull -r
	make
else
	./deploy.sh
endif

update-pot: src/areas.rs src/cache.rs src/util.rs src/webframe.rs src/wsgi.rs src/wsgi_additional.rs Makefile
	xtr --keyword=tr --charset UTF-8 -o po/osm-gimmisn.pot $(filter %.rs,$^)

update-po: po/osm-gimmisn.pot Makefile
	msgmerge --update po/hu/osm-gimmisn.po po/osm-gimmisn.pot

locale/hu/LC_MESSAGES/osm-gimmisn.mo: po/hu/osm-gimmisn.po Makefile
	$(QUIET_MSGFMT)msgfmt --check --statistics --output-file=$@ $<

tags:
	rusty-tags vi
	ln -sf rusty-tags.vi tags

.PHONY: tags
