cd target/debug
hypothermia.exe --pack-resources
cd..
cd..

:: Clear the two other directories
:: rmdir .\target\debug\packed-resources\ /q / s
:: mkdir .\target\debug\packed-resources\
:: rmdir .\target\release\packed-resources\ /q / s
:: mkdir .\target\release\packed-resources\

robocopy .\src\packed-resources\ target\debug\packed-resources\ /mir
robocopy .\src\packed-resources\ target\release\packed-resources\ /mir
pause