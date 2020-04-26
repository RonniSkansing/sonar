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

DOING - Implement webhooks for success / failure
DOING - Refactor config to use builder pattern, perhaps also check out &str usage possible

- Implement rest endpoint for reading log
- Implement a single target mode
- Implement counters for total request in process
- Implement a cluster master / slave node mode
- Improve error messages / maybe change to a tracing lib
- Refactor
- Write a Features part of readme
- Add tests
- Implement metrics for total outgoing requests per second and averaege request time
- Add a 'spread' strategy for dispatching each target at different times, to minimize the overlap of two different targets requesting at the same time
- Implement using prometheus process metrics, note it wont work with for_self as multiple threads are started and all the threads id's must be collected and used
- Better grafana auto dashboard
