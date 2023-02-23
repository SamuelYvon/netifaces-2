import glob

import nox


@nox.session(python=["3.7", "3.8", "3.9", "3.10", "3.11"])
def tests(session: nox.Session) -> None:
    """Runs pytest"""
    wheel = glob.glob("./dist/*whl")[0]
    session.install(wheel)
    session.install("pytest", "pytest-cov")
    session.run(
        "pytest",
        "--cov=netifaces",
        "--cov-config",
        "pyproject.toml",
        "--cov-report=",
        *session.posargs,
        env={"COVERAGE_FILE": f".coverage.{session.python}"},
    )
    session.notify("coverage")


@nox.session
def coverage(session: nox.Session) -> None:
    """Coverage analysis"""
    session.install("coverage[toml]")
    session.run("coverage", "combine")
    session.run("coverage", "report")
    session.run("coverage", "erase")
