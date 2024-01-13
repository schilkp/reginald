_default:
    just --list

format:
    autopep8 --in-place --recursive src/reginald/

lint:
    flake8 src/reginald/

setup_dev:
    python3 -m venv env
    env/bin/pip install -e .[dev]

build:
    -rm -rf dist
    python3 -m build

publish:
    python3 -m twine upload  dist/*
