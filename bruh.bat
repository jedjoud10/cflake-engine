@echo off
setlocal enabledelayedexpansion

:loop
rem Enter the command you want to execute repeatedly below
start cargo run --example mesh --release

timeout /t 6 /nobreak

taskkill /f /im mesh.exe

goto loop

:end
echo Script finished.
pause