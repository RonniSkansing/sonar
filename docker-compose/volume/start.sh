#!/bin/bash
config_file=/opt/sonar/local-mount/sonar.yaml
if [ -f "$FILE" ]; then
    sonar run
else 
    sonar init
    sonar run
fi
