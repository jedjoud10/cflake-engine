:: Just copy all the files to the two packed-resources folders
robocopy .\src\packed-resources\ target\debug\packed-resources\ /mir /njh /njs /ndl /nc /ns
robocopy .\src\packed-resources\ target\release\packed-resources\ /mir /njh /njs /ndl /nc /ns
pause