all: version.py

version.py: .git/$(shell git symbolic-ref HEAD)
	echo "version = '$(shell git describe)'" > $@

check: check-filters check-flake8 check-mypy check-unit check-pylint

check-full: check check-full-pylint

check-full-pylint:
	pylint \
	  --max-line-length=120 \
	  *.py tests/*.py

check-filters: check-filters-syntax check-filters-schema

check-filters-syntax:
	yamllint .travis.yml data/*.yaml

check-flake8:
	flake8 *.py tests/*.py

check-pylint: $(patsubst %.py,%.py.pylinted,$(wildcard *.py tests/*.py))

%.py.pylinted : %.py Makefile
	pylint \
	  --max-line-length=120 \
	  --disable=missing-docstring,fixme,invalid-name,too-few-public-methods,global-statement,too-many-locals \
	  $< && touch $@

check-mypy: version.py
	mypy *.py tests/*.py

check-unit:
	coverage run --branch --module unittest discover tests
	coverage report --show-missing --fail-under=100

check-filters-schema: $(patsubst %.yaml,%.validyaml,$(wildcard data/housenumber-filters-*.yaml))

%.validyaml : %.yaml
	yamale -s data/housenumber-filters.schema.yaml $< && touch $@

server:
	@echo 'Open <http://localhost:8000/osm> in your browser.'
	uwsgi --plugins http,python3 --http :8000 --wsgi-file wsgi.py
