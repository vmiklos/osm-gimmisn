all: version.py

version.py: .git/$(shell git symbolic-ref HEAD) Makefile
	echo '"""The version module allows tracking the last reload of the app server."""' > $@
	echo "VERSION = '$(shell git describe)'" >> $@
	echo "GIT_DIR = '$(shell pwd)'" >> $@

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
	  --disable=too-few-public-methods,too-many-locals \
	  $< && touch $@

check-mypy: version.py
	mypy *.py tests/*.py

check-unit:
	coverage run --branch --module unittest tests/test_helpers.py
	coverage report --show-missing --fail-under=100 helpers.py tests/test_helpers.py

check-filters-schema: $(patsubst %.yaml,%.validyaml,$(wildcard data/housenumber-filters-*.yaml))

%.validyaml : %.yaml
	yamale -s data/housenumber-filters.schema.yaml $< && touch $@

server:
	@./wsgi.py

deploy-pythonanywhere:
	git pull -r
	make
	touch /var/www/vmiklos_pythonanywhere_com_wsgi.py
