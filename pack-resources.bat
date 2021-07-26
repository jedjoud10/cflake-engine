cd target/debug
hypothermia.exe --pack-resources
cd..
cd..
xcopy .\src\packed-resources\ target\debug\packed-resources\ /E /Y
xcopy .\src\packed-resources\ target\release\packed-resources\ /E /Y
pause