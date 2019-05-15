all: version.py

version.py: .git/$(shell git symbolic-ref HEAD)
	echo "version = '$(shell git describe)'" > $@

check: check-filters check-flake8 check-mypy check-unit check-pylint

check-full: check check-full-pylint check-full-filters

check-full-pylint:
	pylint \
	  --max-line-length=120 \
	  *.py tests/*.py

check-full-filters:
	yamllint -c .yamllint.strict .travis.yml data/*.yaml

check-filters: check-filters-syntax check-filters-schema

check-filters-syntax:
	yamllint data/relations.yaml data/housenumber-filters-*.yaml .travis.yml

check-flake8:
	flake8 *.py tests/*.py

check-pylint:
	pylint \
	  --max-line-length=120 \
	  --disable=missing-docstring,fixme,invalid-name,too-few-public-methods,global-statement,too-many-locals \
	  *.py tests/*.py

check-mypy:
	mypy *.py tests/*.py

check-unit:
	coverage run --branch --module unittest discover tests
	coverage report --show-missing --fail-under=100

check-filters-schema:
	for F in data/housenumber-filters-*.yaml; do \
		yamale -s data/housenumber-filters.schema.yaml $$F \
		  || exit $?; \
	done

server:
	@echo 'Open <http://localhost:8000/osm> in your browser.'
	uwsgi --plugins http,python3 --http :8000 --wsgi-file wsgi.py
