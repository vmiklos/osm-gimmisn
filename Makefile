check:
	flake8 overpass_query.py suspicious_streets.py wsgi.py

server:
	uwsgi --plugins http,python3 --http :8000 --wsgi-file wsgi.py
