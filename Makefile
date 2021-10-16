PYTHON_TEST_OBJECTS = \
	tests/test_accept_language.py \
	tests/test_areas.py \
	tests/test_cache.py \
	tests/test_cache_yamls.py \
	tests/test_context.py \
	tests/test_cron.py \
	tests/test_missing_housenumbers.py \
	tests/test_overpass_query.py \
	tests/test_parse_access_log.py \
	tests/test_validator.py \
	tests/test_webframe.py \
	tests/test_wsgi.py \
	tests/test_wsgi_additional.py \
	tests/test_wsgi_json.py \

# These have good coverage.
PYTHON_SAFE_OBJECTS = \
	api.py \
	areas.py \
	cache.py \
	cache_yamls.py \
	context.py \
	cron.py \
	parse_access_log.py \
	stats.py \
	util.py \
	validator.py \
	webframe.py \
	wsgi.py \

# These have bad coverage.
PYTHON_UNSAFE_OBJECTS = \

PYTHON_OBJECTS = \
	$(PYTHON_TEST_OBJECTS) \
	$(PYTHON_SAFE_OBJECTS) \
	$(PYTHON_UNSAFE_OBJECTS) \

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
	config.ts \
	main.ts \
	stats.ts \
	types.d.ts \

RS_OBJECTS = \
	src/accept_language.rs \
	src/area_files.rs \
	src/areas.rs \
	src/bin/missing_housenumbers.rs \
	src/bin/rouille.rs \
	src/context.rs \
	src/cache.rs \
	src/cron.rs \
	src/i18n.rs \
	src/lib.rs \
	src/missing_housenumbers.rs \
	src/overpass_query.rs \
	src/ranges.rs \
	src/stats.rs \
	src/util.rs \
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
	QUIET_FLAKE8 = @echo '   ' FLAKE8 $@;
	QUIET_MSGFMT = @echo '   ' MSGMFT $@;
	QUIET_PYLINT = @echo '   ' PYLINT $@;
	QUIET_ESLINT = @echo '   ' ESLINT $@;
	QUIET_VALIDATOR = @echo '   ' VALIDATOR $@;
	QUIET_YAMLLINT = @echo '   ' YAMLLINT $@;
endif

all: rust.so builddir/bundle.js css wsgi.ini data/yamls.cache locale/hu/LC_MESSAGES/osm-gimmisn.mo target/${TARGET_PATH}/missing_housenumbers target/${TARGET_PATH}/rouille

clean:
	rm -f config.ts
	rm -f $(patsubst %.yaml,%.yamllint,$(filter-out .github/workflows/tests.yml,$(YAML_OBJECTS)))
	rm -f $(patsubst %.yaml,%.validyaml,$(YAML_SAFE_OBJECTS))
	rm -f $(patsubst %.py,%.flake8,$(PYTHON_OBJECTS))
	rm -f $(patsubst %.py,%.pylint,$(PYTHON_OBJECTS))
	rm -rf $(patsubst %.py,%.mypy,$(PYTHON_OBJECTS)) .mypy_cache
	rm -rf $(patsubst %.ts,%.eslint,$(TS_OBJECTS)) builddir

check: all check-filters check-flake8 check-mypy check-unit check-rustunit check-pylint check-eslint check-rustfmt check-clippy
	@echo "make check: ok"

check-rustfmt: Cargo.toml $(RS_OBJECTS)
	cargo fmt -- --check && touch $@

check-clippy: Cargo.toml $(RS_OBJECTS)
	cargo clippy ${CARGO_OPTIONS} && touch $@

rust.so: target/${TARGET_PATH}/librust.so
	ln -sf target/${TARGET_PATH}/librust.so rust.so

target/${TARGET_PATH}/librust.so: Cargo.toml $(RS_OBJECTS)
	cargo build --lib ${CARGO_OPTIONS}

target/${TARGET_PATH}/missing_housenumbers: Cargo.toml $(RS_OBJECTS)
	cargo build --bin missing_housenumbers ${CARGO_OPTIONS} --no-default-features

target/${TARGET_PATH}/rouille: Cargo.toml $(RS_OBJECTS)
	cargo build --bin rouille ${CARGO_OPTIONS} --no-default-features

check-rustunit: Cargo.toml $(RS_OBJECTS) locale/hu/LC_MESSAGES/osm-gimmisn.mo testdata
	cargo test --lib --no-default-features ${CARGO_OPTIONS} -- --test-threads=1

config.ts: wsgi.ini Makefile
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

data/yamls.cache: cache_yamls.py rust.so $(YAML_OBJECTS)
	./cache_yamls.py data workdir

tests/data/yamls.cache: cache_yamls.py rust.so $(YAML_TEST_OBJECTS)
	./cache_yamls.py tests/data tests/workdir

check-filters: check-filters-syntax check-filters-schema

check-filters-syntax: $(patsubst %.yaml,%.yamllint,$(YAML_OBJECTS))

check-flake8: $(patsubst %.py,%.flake8,$(PYTHON_OBJECTS))

check-pylint: $(patsubst %.py,%.pylint,$(PYTHON_OBJECTS))

check-eslint: $(patsubst %.ts,%.eslint,$(TS_OBJECTS))

check-mypy: $(PYTHON_OBJECTS)
	env PYTHONPATH=.:tests mypy --python-version 3.6 --strict --no-error-summary . && touch $@

%.pylint : %.py Makefile .pylintrc rust.so
	$(QUIET_PYLINT)env PYTHONPATH=. pylint $< && touch $@

%.eslint : %.ts Makefile .eslintrc
	$(QUIET_ESLINT)node_modules/.bin/eslint $< && touch $@

%.flake8: %.py Makefile setup.cfg
	$(QUIET_FLAKE8)flake8 $< && touch $@

check-unit: rust.so data/yamls.cache testdata
	env PYTHONPATH=.:tests coverage run --branch --module unittest $(PYTHON_TEST_OBJECTS)
	env PYTHONPATH=.:tests coverage report --show-missing --fail-under=100 $(PYTHON_SAFE_OBJECTS)

check-filters-schema: $(patsubst %.yaml,%.validyaml,$(YAML_SAFE_OBJECTS))

%.validyaml : %.yaml validator.py
	$(QUIET_VALIDATOR)./validator.py $< && touch $@

%.yamllint : %.yaml Makefile .yamllint
	$(QUIET_YAMLLINT)yamllint --strict $< && touch $@

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
