:: Just copy all the files to the two packed-resources folders
robocopy %1 target\debug\packed-resources\ /mir /njh /njs /ndl /nc /ns
robocopy %1 target\release\packed-resources\ /mir /njh /njs /ndl /nc /ns
pause