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
- init from file with a domain per line
- Implement webhooks for success / failure
- Implement rest endpoint for reading log
- Improve error messages
- Refactor
- Add tests
- Add a 'spread' strategy for dispatching each target at different times, to minimize the overlap of two different targets requesting at the same time
- Implement using prometheus process metrics, note it wont work with for_self as multiple threads are started and all the threads id's must be collected and used
