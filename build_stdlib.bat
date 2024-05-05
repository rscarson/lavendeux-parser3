@echo off
REM Copyright Richard Carson, 2024
REM Licensed under the MIT License

REM This batch file compiles the Lavendeux standard library.
REM The library will not include debug information unless the --debug flag is passed.
REM The production build should NOT include debug information.
REM Usage: compile.bat [--debug]

SET "DEBUG_FLAG="

REM Check if --debug flag is passed
if "%1"=="--debug" (
    SET "DEBUG_FLAG=-D"
)

cargo run --bin compiler -- -F -f stdlib/src/stdlib.lav -o stdlib/stdlib.lbc --allow-syscalld %DEBUG_FLAG%