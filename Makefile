PYTHON_TEST_OBJECTS = \
	tests/test_get_reference_housenumbers.py \
	tests/test_get_reference_streets.py \
	tests/test_helpers.py \
	tests/test_overpass_query.py \
	tests/test_validator.py \

# These have good coverage.
PYTHON_SAFE_OBJECTS = \
	get_reference_housenumbers.py \
	get_reference_streets.py \
	helpers.py \
	overpass_query.py \
	validator.py \

PYTHON_OBJECTS = \
	$(PYTHON_TEST_OBJECTS) \
	$(PYTHON_SAFE_OBJECTS) \
	cron.py \
	missing_housenumbers.py \
	missing_streets.py \
	version.py \
	wsgi.py \
	i18n.py \

all: version.py po/hu/osm-gimmisn.mo

clean:
	rm -f version.py
	rm -f $(patsubst %.yaml,%.yamllint,$(wildcard data/relations.yaml data/relation-*.yaml))
	rm -f $(patsubst %.yaml,%.validyaml,$(wildcard data/relations.yaml data/relation-*.yaml))
	rm -f $(patsubst %.py,%.flake8,$(PYTHON_OBJECTS))
	rm -f $(patsubst %.py,%.pylint,$(PYTHON_OBJECTS))
	rm -f $(patsubst %.py,%.mypy,$(PYTHON_OBJECTS))

check: all check-filters check-flake8 check-mypy check-unit check-pylint

version.py: .git/$(shell git symbolic-ref HEAD) Makefile
	echo '"""The version module allows tracking the last reload of the app server."""' > $@
	echo "VERSION = '$(shell git describe)'" >> $@
	echo "GIT_DIR = '$(shell pwd)'" >> $@

check-filters: check-filters-syntax check-filters-schema

check-filters-syntax: $(patsubst %.yaml,%.yamllint,$(wildcard data/relations.yaml data/relation-*.yaml))

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

check-filters-schema: $(patsubst %.yaml,%.validyaml,$(wildcard data/relations.yaml data/relation-*.yaml))

%.validyaml : %.yaml validator.py
	./validator.py $< && touch $@

%.yamllint : %.yaml
	yamllint $< && touch $@

server:
	./wsgi.py

deploy-pythonanywhere:
	git pull -r
	make
	touch /var/www/vmiklos_pythonanywhere_com_wsgi.py

po/osm-gimmisn.pot: helpers.py wsgi.py Makefile
	xgettext --keyword=_ --language=Python --add-comments --sort-output --from-code=UTF-8 -o $@ $(filter %.py,$^)

po/hu/osm-gimmisn.po: po/osm-gimmisn.pot Makefile
	msgmerge --update $@ $<

po/hu/osm-gimmisn.mo: po/hu/osm-gimmisn.po Makefile
	msgfmt --output-file=$@ $<
