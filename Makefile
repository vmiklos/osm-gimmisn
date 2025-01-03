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
	src/area_files.rs \
	src/areas.rs \
	src/areas/tests.rs \
	src/cache.rs \
	src/cache/tests.rs \
	src/cache_yamls.rs \
	src/cache_yamls/tests.rs \
	src/context.rs \
	src/context/system.rs \
	src/context/tests.rs \
	src/cron.rs \
	src/cron/tests.rs \
	src/i18n.rs \
	src/i18n/tests.rs \
	src/lib.rs \
	src/main.rs \
	src/missing_housenumbers.rs \
	src/missing_housenumbers/tests.rs \
	src/overpass_query.rs \
	src/overpass_query/tests.rs \
	src/parse_access_log.rs \
	src/parse_access_log/tests.rs \
	src/ranges.rs \
	src/ranges/tests.rs \
	src/serde.rs \
	src/sql.rs \
	src/stats.rs \
	src/stats/tests.rs \
	src/sync_ref.rs \
	src/sync_ref/tests.rs \
	src/util.rs \
	src/util/tests.rs \
	src/validator.rs \
	src/validator/tests.rs \
	src/webframe.rs \
	src/webframe/tests.rs \
	src/wsgi.rs \
	src/wsgi/tests.rs \
	src/wsgi_additional.rs \
	src/wsgi_additional/tests.rs \
	src/wsgi_json.rs \
	src/wsgi_json/tests.rs \
	src/yattag.rs \
	src/yattag/tests.rs \

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
	QUIET_WEBPACK = @echo '   ' WEBPACK $@;
	QUIET_VALIDATOR = @echo '   ' VALIDATOR $@;
endif

all: target/browser/bundle.js css workdir/wsgi.ini data/yamls.cache locale/hu/LC_MESSAGES/osm-gimmisn.mo target/${TARGET_PATH}/osm-gimmisn

clean:
	rm -rf target
	rm -f $(patsubst %.yaml,%.validyaml,$(YAML_SAFE_OBJECTS))
	rm -f config.ts $(patsubst %.ts,%.eslint,$(TS_OBJECTS))

check: all check-filters check-unit check-eslint check-rustfmt check-clippy
	@echo "make check: ok"

check-rustfmt: Cargo.toml $(RS_OBJECTS)
	cargo fmt -- --check && touch $@

check-clippy: Cargo.toml .github/workflows/tests.yml $(RS_OBJECTS)
	cargo clippy ${CARGO_OPTIONS} && touch $@

target/${TARGET_PATH}/osm-gimmisn: $(RS_OBJECTS) Cargo.toml Makefile
	cargo build ${CARGO_OPTIONS}

# Without coverage: cargo test --lib
check-unit: Cargo.toml $(RS_OBJECTS) locale/hu/LC_MESSAGES/osm-gimmisn.mo data/yamls.cache
	cargo llvm-cov --lib -q --ignore-filename-regex '(serde|system).rs' --show-missing-lines --fail-under-lines 100 ${CARGO_OPTIONS} -- --test-threads=1

src/browser/config.ts: workdir/wsgi.ini Makefile
	printf 'const uriPrefix = "%s";\nexport { uriPrefix };\n' $(shell (grep uri_prefix workdir/wsgi.ini || echo "/osm") |sed 's/uri_prefix = //') > $@

ifdef TSDEBUG
WEBPACK_OPTIONS = --mode=development --devtool inline-source-map
else
WEBPACK_OPTIONS = --mode=production
endif

target/browser/bundle.js: $(TS_OBJECTS) package-lock.json Makefile
	mkdir -p target/browser
	$(QUIET_WEBPACK)npx webpack ${WEBPACK_OPTIONS} --config webpack.config.js
	touch $@

package-lock.json: package.json
	npm install
	touch $@

css: target/browser/osm.min.css

target/browser/osm.min.css: static/osm.css package-lock.json
	mkdir -p workdir
	[ -x "./node_modules/.bin/cleancss" ] && npx cleancss -o $@ $< || cp -a $< $@

# Intentionally don't update this when the source changes.
workdir/wsgi.ini:
	mkdir -p workdir
	cp data/wsgi.ini.template workdir/wsgi.ini

data/yamls.cache: target/${TARGET_PATH}/osm-gimmisn $(YAML_OBJECTS)
	target/${TARGET_PATH}/osm-gimmisn cache-yamls data workdir

check-eslint: $(patsubst %.ts,%.eslint,$(TS_OBJECTS))

%.eslint : %.ts Makefile eslint.config.mjs package-lock.json
	$(QUIET_ESLINT)npx eslint $< && touch $@

check-filters: $(patsubst %.yaml,%.validyaml,$(YAML_SAFE_OBJECTS))

%.validyaml : %.yaml target/${TARGET_PATH}/osm-gimmisn
	$(QUIET_VALIDATOR)target/${TARGET_PATH}/osm-gimmisn validator $< && touch $@

run: all
	target/${TARGET_PATH}/osm-gimmisn rouille

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

guide:
	cd guide && mdbook build
	cargo about generate about.hbs > guide/book/license.html

run-guide:
	cd guide && mdbook serve --hostname 127.0.0.1

.PHONY: tags guide
