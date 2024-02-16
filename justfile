PY := "python"
PYENV := "env/bin/python"

_default:
    just --list

format:
    {{PYENV}} -m autopep8 --in-place --recursive src/reginald/

lint:
    {{PYENV}} -m flake8 src/reginald/

setup_dev:
    {{PY}} -m venv env
    {{PYENV}} -m pip install -e .[dev]

build:
    -rm -rf dist
    {{PYENV}} -m build

publish:
    {{PYENV}} -m twine upload  dist/*
