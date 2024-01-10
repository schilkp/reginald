_default:
    just --list

format:
    autopep8 --in-place --recursive run.py reginald/

lint:
    flake8 reginald/ run.py
