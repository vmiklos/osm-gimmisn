PYTHON_OBJECTS = \
	cron.py \
	get_reference_housenumbers.py \
	get_reference_streets.py \
	helpers.py \
	overpass_query.py \
	suspicious_relations.py \
	suspicious_streets.py \
	version.py \
	wsgi.py \
	tests/test_helpers.py \

all: version.py

check: all check-filters check-flake8 check-mypy check-unit check-pylint

version.py: .git/$(shell git symbolic-ref HEAD) Makefile
	echo '"""The version module allows tracking the last reload of the app server."""' > $@
	echo "VERSION = '$(shell git describe)'" >> $@
	echo "GIT_DIR = '$(shell pwd)'" >> $@

check-filters: check-filters-syntax check-filters-schema

check-filters-syntax: $(patsubst %.yaml,%.yamllint,$(wildcard .travis.yml data/relation-*.yaml))

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
	coverage run --branch --module unittest tests/test_helpers.py
	coverage report --show-missing --fail-under=100 helpers.py tests/test_helpers.py

check-filters-schema: $(patsubst %.yaml,%.validyaml,$(wildcard data/relation-*.yaml))

%.validyaml : %.yaml
	yamale -s data/relation.schema.yaml $< && touch $@

%.yamllint : %.yaml
	yamllint $< && touch $@

server:
	./wsgi.py

deploy-pythonanywhere:
	git pull -r
	make
	touch /var/www/vmiklos_pythonanywhere_com_wsgi.py
