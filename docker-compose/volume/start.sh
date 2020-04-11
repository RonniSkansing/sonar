#!/bin/bash
config_file=/opt/sonar/local-mount/sonar.yaml
if [ -f "$FILE" ]; then
    sonar -d run
else 
    sonar init
    sonar -d run
fi
