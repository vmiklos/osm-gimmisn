PYTHON_TEST_OBJECTS = \
	tests/test_accept_language.py \
	tests/test_areas.py \
	tests/test_cache_yamls.py \
	tests/test_cherry.py \
	tests/test_config.py \
	tests/test_cron.py \
	tests/test_get_reference_housenumbers.py \
	tests/test_get_reference_streets.py \
	tests/test_i18n.py \
	tests/test_missing_housenumbers.py \
	tests/test_missing_streets.py \
	tests/test_overpass_query.py \
	tests/test_parse_access_log.py \
	tests/test_ranges.py \
	tests/test_stats.py \
	tests/test_util.py \
	tests/test_validator.py \
	tests/test_webframe.py \
	tests/test_wsgi.py \
	tests/test_wsgi_additional.py \
	tests/test_wsgi_json.py \

# These have good coverage.
PYTHON_SAFE_OBJECTS = \
	accept_language.py \
	areas.py \
	cache_yamls.py \
	cherry.py \
	config.py \
	cron.py \
	get_reference_housenumbers.py \
	get_reference_streets.py \
	i18n.py \
	missing_housenumbers.py \
	missing_streets.py \
	overpass_query.py \
	parse_access_log.py \
	ranges.py \
	stats.py \
	util.py \
	validator.py \
	version.py \
	webframe.py \
	wsgi.py \
	wsgi_additional.py \
	wsgi_json.py \

# These have bad coverage.
PYTHON_UNSAFE_OBJECTS = \
	additional_streets.py \
	invalid_refstreets.py \

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

ifndef V
	QUIET_FLAKE8 = @echo '   ' FLAKE8 $@;
	QUIET_MSGFMT = @echo '   ' MSGMFT $@;
	QUIET_MYPY = @echo '   ' MYPY $@;
	QUIET_PYLINT = @echo '   ' PYLINT $@;
	QUIET_ESLINT = @echo '   ' ESLINT $@;
	QUIET_VALIDATOR = @echo '   ' VALIDATOR $@;
	QUIET_YAMLLINT = @echo '   ' YAMLLINT $@;
endif

all: version.py workdir/bundle.js css wsgi.ini data/yamls.pickle locale/hu/LC_MESSAGES/osm-gimmisn.mo

clean:
	rm -f version.py config.ts
	rm -f $(patsubst %.yaml,%.yamllint,$(filter-out .github/workflows/tests.yml,$(YAML_OBJECTS)))
	rm -f $(patsubst %.yaml,%.validyaml,$(YAML_SAFE_OBJECTS))
	rm -f $(patsubst %.py,%.flake8,$(PYTHON_OBJECTS))
	rm -f $(patsubst %.py,%.pylint,$(PYTHON_OBJECTS))
	rm -f $(patsubst %.py,%.mypy,$(PYTHON_OBJECTS))
	rm -f $(patsubst %.ts,%.eslint,$(TS_OBJECTS))

check: all check-filters check-flake8 check-mypy check-unit check-pylint check-eslint

version.py: .git/$(shell git symbolic-ref HEAD) Makefile
	$(file > $@,"""The version module allows tracking the last reload of the app server.""")
	$(file >> $@,VERSION = '$(shell git describe --tags)')

config.ts: wsgi.ini Makefile
	printf 'const uriPrefix = "%s";\nexport { uriPrefix };\n' $(shell grep prefix wsgi.ini |sed 's/uri_prefix = //') > $@

ifdef TSDEBUG
BROWSERIFY_OPTIONS = --debug
else
BROWSERIFY_OPTIONS = --plugin tinyify
endif
BROWSERIFY_OPTIONS += --plugin tsify

workdir/bundle.js: $(TS_OBJECTS) package-lock.json
	node_modules/.bin/browserify -o workdir/bundle.js $(BROWSERIFY_OPTIONS) $(TS_OBJECTS)

package-lock.json: package.json
	npm install
	touch package-lock.json

css: workdir/osm.min.css

workdir/osm.min.css: static/osm.css package-lock.json
	./node_modules/.bin/cleancss -o $@ $<

tests/workdir/osm.min.css: workdir/osm.min.css
	cp -a $< $@

# Intentionally don't update this when the source changes.
wsgi.ini:
	cp data/wsgi.ini.template wsgi.ini

data/yamls.pickle: cache_yamls.py $(YAML_OBJECTS)
	./cache_yamls.py data workdir

tests/data/yamls.pickle: cache_yamls.py $(YAML_TEST_OBJECTS)
	./cache_yamls.py tests/data tests/workdir

check-filters: check-filters-syntax check-filters-schema

check-filters-syntax: $(patsubst %.yaml,%.yamllint,$(YAML_OBJECTS))

check-flake8: $(patsubst %.py,%.flake8,$(PYTHON_OBJECTS))

check-pylint: $(patsubst %.py,%.pylint,$(PYTHON_OBJECTS))

check-eslint: $(patsubst %.ts,%.eslint,$(TS_OBJECTS))

check-mypy: $(patsubst %.py,%.mypy,$(PYTHON_OBJECTS))

%.pylint : %.py Makefile .pylintrc
	$(QUIET_PYLINT)env PYTHONPATH=. pylint $< && touch $@

%.eslint : %.ts Makefile .eslintrc
	$(QUIET_ESLINT)node_modules/.bin/eslint $< && touch $@

%.mypy: %.py Makefile
	$(QUIET_MYPY)mypy --python-version 3.6 --strict --no-error-summary $< && touch $@

%.flake8: %.py Makefile
	$(QUIET_FLAKE8)flake8 $< && touch $@

check-unit: version.py data/yamls.pickle tests/data/yamls.pickle tests/workdir/osm.min.css
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
	cd $(HOME) && $(PWD)/wsgi.py

deploy:
ifeq (,$(wildcard ./deploy.sh))
	git pull -r
	make
else
	./deploy.sh
endif

update-pot: areas.py webframe.py wsgi.py wsgi_additional.py util.py Makefile
	xgettext --keyword=_ --language=Python --add-comments --sort-output --from-code=UTF-8 -o po/osm-gimmisn.pot $(filter %.py,$^)

update-po: po/osm-gimmisn.pot Makefile
	msgmerge --update po/hu/osm-gimmisn.po po/osm-gimmisn.pot

locale/hu/LC_MESSAGES/osm-gimmisn.mo: po/hu/osm-gimmisn.po Makefile
	$(QUIET_MSGFMT)msgfmt --check --statistics --output-file=$@ $<

tags:
	ctags --python-kinds=-iv --fields=+l --extra=+q -R --totals=yes *
