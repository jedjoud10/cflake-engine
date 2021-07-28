:: Clear the main directory
rmdir .\src\packed-resources\ /q / s
mkdir .\src\packed-resources\
cd target/debug
hypothermia.exe --pack-resources
cd..
cd..

:: Clear the two other directories
rmdir .\target\debug\packed-resources\ /q / s
mkdir .\target\debug\packed-resources\
rmdir .\target\release\packed-resources\ /q / s
mkdir .\target\release\packed-resources\

xcopy .\src\packed-resources\ target\debug\packed-resources\ /E /Y
xcopy .\src\packed-resources\ target\release\packed-resources\ /E /Y
pause