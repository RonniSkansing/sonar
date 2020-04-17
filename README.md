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
