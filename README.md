# Internal Event Flow

- Sonar started

  - Job: WebServer Started with prometheus stats exporter
  - Event: Config created/changed/deleted
    - Job: Create new grafana dashboard
      - 3.Party: Grafana reads new dashboard
    - Job: Change Grafana prometheus exporter
    - Job: Stop / Start / Change Requesters

- Event: Requester completed/failed request
  - Job: Logger writes to log file
  - Job: Grafana metrics added to exporter client

* TODO

- TODO's
- Implement Async file writers
- Implement webhooks for success / failure
- Implement rest endpoint for reading log
- Implement using prometheus process metrics, note it wont work with for_self as multiple threads are started and all the threads id's must be collected and used
