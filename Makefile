check-full: check
	pylint \
	  --max-line-length=120 \
	  *.py tests/*.py

check:
	yamllint data/*.yaml
	shellcheck *.sh
	flake8 *.py tests/*.py
	pylint \
	  --max-line-length=120 \
	  --disable=missing-docstring,fixme,invalid-name,too-few-public-methods,global-statement \
	  *.py tests/*.py
	pycodestyle --max-line-length=120 *.py tests/*.py
	python3 -m unittest discover tests

server:
	@echo 'Open <http://localhost:8000/osm> in your browser.'
	uwsgi --plugins http,python3 --http :8000 --wsgi-file wsgi.py
