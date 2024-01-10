export const TELEGRAF_PROMETHEUS_CODE = `kind: Metric
id: telegraf_prometheus_nginx_connections_waiting
collector: telegraf
metadata:
  inputs:
    prometheus:
      - urls: [http://0.0.0.0:9090/metrics]
        period: 1m
        delay: 0s
        interval: 1m
        # This metric represents the wating number of connections that the nginx reserves. If this value remains high, it signifies that the nginx is using a significant amount of connections, which could trigger a scale-out.
        namepass: [nginx_connections_waiting] 
  outputs:
    wave-autoscale: {}
`;
