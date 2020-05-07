PYTHON_TEST_OBJECTS = \
	tests/test_accept_language.py \
	tests/test_areas.py \
	tests/test_cache_yamls.py \
	tests/test_cherry.py \
	tests/test_cron.py \
	tests/test_get_reference_housenumbers.py \
	tests/test_get_reference_streets.py \
	tests/test_i18n.py \
	tests/test_missing_housenumbers.py \
	tests/test_missing_streets.py \
	tests/test_overpass_query.py \
	tests/test_ranges.py \
	tests/test_util.py \
	tests/test_validator.py \
	tests/test_webframe.py \
	tests/test_wsgi.py \

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
	ranges.py \
	util.py \
	validator.py \
	version.py \
	webframe.py \
	wsgi.py \

PYTHON_OBJECTS = \
	$(PYTHON_TEST_OBJECTS) \
	$(PYTHON_SAFE_OBJECTS) \
	stats.py \

# These are valid.
YAML_SAFE_OBJECTS = \
	$(wildcard data/relation-*.yaml) \
	data/relations.yaml \

# These are well-formed.
YAML_OBJECTS = \
	$(YAML_SAFE_OBJECTS) \
	.travis.yml \
	data/refcounty-names.yaml \
	data/refsettlement-names.yaml \

YAML_TEST_OBJECTS = \
	$(wildcard tests/data/relation-*.yaml) \
	tests/data/relations.yaml \
	tests/data/refcounty-names.yaml \
	tests/data/refsettlement-names.yaml \

ifndef V
	QUIET_FLAKE8 = @echo '   ' FLAKE8 $@;
	QUIET_MSGFMT = @echo '   ' MSGMFT $@;
	QUIET_MYPY = @echo '   ' MYPY $@;
	QUIET_PYLINT = @echo '   ' PYLINT $@;
	QUIET_VALIDATOR = @echo '   ' VALIDATOR $@;
	QUIET_YAMLLINT = @echo '   ' YAMLLINT $@;
endif

all: version.py data/yamls.pickle locale/hu/LC_MESSAGES/osm-gimmisn.mo

clean:
	rm -f version.py
	rm -f $(patsubst %.yaml,%.yamllint,$(filter-out .travis.yml,$(YAML_OBJECTS)))
	rm -f $(patsubst %.yaml,%.validyaml,$(YAML_SAFE_OBJECTS))
	rm -f $(patsubst %.py,%.flake8,$(PYTHON_OBJECTS))
	rm -f $(patsubst %.py,%.pylint,$(PYTHON_OBJECTS))
	rm -f $(patsubst %.py,%.mypy,$(PYTHON_OBJECTS))

check: all check-filters check-flake8 check-mypy check-unit check-pylint

version.py: .git/$(shell git symbolic-ref HEAD) Makefile
	$(file > $@,"""The version module allows tracking the last reload of the app server.""")
	$(file >> $@,VERSION = '$(shell git describe --tags)')

data/yamls.pickle: $(YAML_OBJECTS)
	./cache_yamls.py data

tests/data/yamls.pickle: $(YAML_TEST_OBJECTS)
	./cache_yamls.py tests/data

check-filters: check-filters-syntax check-filters-schema

check-filters-syntax: $(patsubst %.yaml,%.yamllint,$(YAML_OBJECTS))

check-flake8: $(patsubst %.py,%.flake8,$(PYTHON_OBJECTS))

check-pylint: $(patsubst %.py,%.pylint,$(PYTHON_OBJECTS))

check-mypy: $(patsubst %.py,%.mypy,$(PYTHON_OBJECTS))

# pylint itself raises some warnings, ignore them.
%.pylint : %.py Makefile .pylintrc
	$(QUIET_PYLINT)pylint $< && touch $@

%.mypy: %.py Makefile
	$(QUIET_MYPY)mypy --python-version 3.6 --strict --no-error-summary $< && touch $@

%.flake8: %.py Makefile
	$(QUIET_FLAKE8)flake8 $< && touch $@

check-unit: tests/data/yamls.pickle
	coverage run --branch --module unittest $(PYTHON_TEST_OBJECTS)
	coverage report --show-missing --fail-under=100 $(PYTHON_SAFE_OBJECTS)

check-filters-schema: $(patsubst %.yaml,%.validyaml,$(YAML_SAFE_OBJECTS))

%.validyaml : %.yaml validator.py
	$(QUIET_VALIDATOR)./validator.py $< && touch $@

%.yamllint : %.yaml Makefile .yamllint
	$(QUIET_YAMLLINT)yamllint $< && touch $@

# Make sure that the current directory is *not* the repo root but something else to catch
# non-absolute paths.
server:
	cd $(HOME) && $(PWD)/wsgi.py

deploy:
	git pull -r
	make

update-pot: areas.py webframe.py wsgi.py util.py Makefile
	xgettext --keyword=_ --language=Python --add-comments --sort-output --from-code=UTF-8 -o po/osm-gimmisn.pot $(filter %.py,$^)

update-po: po/osm-gimmisn.pot Makefile
	msgmerge --update po/hu/osm-gimmisn.po po/osm-gimmisn.pot

locale/hu/LC_MESSAGES/osm-gimmisn.mo: po/hu/osm-gimmisn.po Makefile
	$(QUIET_MSGFMT)msgfmt --check --statistics --output-file=$@ $<

tags:
	ctags --python-kinds=-iv --fields=+l --extra=+q -R --totals=yes *
