server:
	uwsgi --plugins http,python3 --http :8000 --wsgi-file wsgi.py
