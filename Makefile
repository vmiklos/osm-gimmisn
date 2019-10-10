PYTHON_TEST_OBJECTS = \
	tests/test_accept_language.py \
	tests/test_get_reference_housenumbers.py \
	tests/test_get_reference_streets.py \
	tests/test_helpers.py \
	tests/test_overpass_query.py \
	tests/test_util.py \
	tests/test_validator.py \

# These have good coverage.
PYTHON_SAFE_OBJECTS = \
	accept_language.py \
	get_reference_housenumbers.py \
	get_reference_streets.py \
	helpers.py \
	overpass_query.py \
	util.py \
	validator.py \

PYTHON_OBJECTS = \
	$(PYTHON_TEST_OBJECTS) \
	$(PYTHON_SAFE_OBJECTS) \
	cron.py \
	i18n.py \
	missing_housenumbers.py \
	missing_streets.py \
	version.py \
	wsgi.py \

# These are valid.
YAML_SAFE_OBJECTS = \
	$(wildcard data/relation-*.yaml) \
	data/relations.yaml \

# These are well-formed.
YAML_OBJECTS = \
	$(YAML_SAFE_OBJECTS) \
	.travis.yml \
	data/refmegye-names.yaml \
	data/reftelepules-names.yaml \

all: version.py locale/hu/LC_MESSAGES/osm-gimmisn.mo

clean:
	rm -f version.py
	rm -f $(patsubst %.yaml,%.yamllint,$(YAML_OBJECTS))
	rm -f $(patsubst %.yaml,%.validyaml,$(YAML_SAFE_OBJECTS))
	rm -f $(patsubst %.py,%.flake8,$(PYTHON_OBJECTS))
	rm -f $(patsubst %.py,%.pylint,$(PYTHON_OBJECTS))
	rm -f $(patsubst %.py,%.mypy,$(PYTHON_OBJECTS))

check: all check-filters check-flake8 check-mypy check-unit check-pylint

version.py: .git/$(shell git symbolic-ref HEAD) Makefile
	echo '"""The version module allows tracking the last reload of the app server."""' > $@
	echo "VERSION = '$(shell git describe)'" >> $@
	echo "GIT_DIR = '$(shell pwd)'" >> $@

check-filters: check-filters-syntax check-filters-schema

check-filters-syntax: $(patsubst %.yaml,%.yamllint,$(YAML_OBJECTS))

check-flake8: $(patsubst %.py,%.flake8,$(PYTHON_OBJECTS))

check-pylint: $(patsubst %.py,%.pylint,$(PYTHON_OBJECTS))

check-mypy: $(patsubst %.py,%.mypy,$(PYTHON_OBJECTS))

%.pylint : %.py Makefile .pylintrc
	pylint $< && touch $@

%.mypy: %.py Makefile
	mypy --python-version 3.5 --strict $< && touch $@

%.flake8: %.py Makefile
	flake8 $< && touch $@

check-unit:
	coverage run --branch --module unittest $(PYTHON_TEST_OBJECTS)
	coverage report --show-missing --fail-under=100 $(PYTHON_SAFE_OBJECTS)

check-filters-schema: $(patsubst %.yaml,%.validyaml,$(YAML_SAFE_OBJECTS))

%.validyaml : %.yaml validator.py
	./validator.py $< && touch $@

%.yamllint : %.yaml
	yamllint $< && touch $@

# Make sure that the current directory is *not* the repo root but the home directory, this matches
# the environment of the PythonAnywhere instance.
server:
	cd $(HOME) && $(PWD)/wsgi.py

deploy-pythonanywhere:
	git pull -r
	make
	touch /var/www/vmiklos_pythonanywhere_com_wsgi.py

update-pot: helpers.py wsgi.py Makefile
	xgettext --keyword=_ --language=Python --add-comments --sort-output --from-code=UTF-8 -o po/osm-gimmisn.pot $(filter %.py,$^)

update-po: po/osm-gimmisn.pot Makefile
	msgmerge --update po/hu/osm-gimmisn.po po/osm-gimmisn.pot

locale/hu/LC_MESSAGES/osm-gimmisn.mo: po/hu/osm-gimmisn.po Makefile
	msgfmt --check --statistics --output-file=$@ $<
