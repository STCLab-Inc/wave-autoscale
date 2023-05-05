import { CloudwatchDataMetricMetadata } from '@/types/bindings/cloudwatch-data-metric';
import { CloudwatchStatisticsMetricMetadata } from '@/types/bindings/cloudwatch-statistics-metric';
import { MetricDefinition } from '@/types/bindings/metric-definition';
import { PrometheusMetricMetadata } from '@/types/bindings/prometheus-metric';

// https://stackoverflow.com/questions/51465182/typescript-get-keys-of-a-generic-interface-without-using-enum
// type KeyTypes<T> = { [K in keyof T]: { key: K; type: T[K] } }[keyof T][];
export type KeyType = { key: string; type: string };
function getInterfaceKeyTypes<T extends {}>(obj: T): KeyType[] {
  const keys = Object.keys(obj) as (keyof T)[];
  return keys.map((key) => ({
    key: key as string,
    type: typeof obj[key],
  })) as KeyType[];
}

export type MetricKeyType = { metricName: string; keyTypes: KeyType[] };

export function getMetricKeyTypes(): MetricKeyType[] {
  const metricKeyTypes: MetricKeyType[] = [];

  // Prometheus
  const prometheusMetric = getInterfaceKeyTypes<PrometheusMetricMetadata>({
    query: '',
    polling_interval: 0,
    prometheus_token: '',
    prometheus_url: '',
  });
  metricKeyTypes.push({
    metricName: 'prometheus',
    keyTypes: prometheusMetric,
  });

  // Cloudwatch Data
  const cloudwatchDataMetric =
    getInterfaceKeyTypes<CloudwatchDataMetricMetadata>({
      access_key: '',
      secret_key: '',
      region: '',
      expression: '',
      polling_interval: 0,
      duration_seconds: 0,
      period: 0,
    });
  metricKeyTypes.push({
    metricName: 'cloudwatch-data',
    keyTypes: cloudwatchDataMetric,
  });

  // Cloudwatch Statistics
  const cloudwatchStatisticsMetric =
    getInterfaceKeyTypes<CloudwatchStatisticsMetricMetadata>({
      access_key: '',
      secret_key: '',
      region: '',
      namespace: '',
      metric_name: '',
      statistic: '',
      polling_interval: 0,
      duration_seconds: 0,
      period: 0,
      dimensions: [],
      unit: '',
    });
  metricKeyTypes.push({
    metricName: 'cloudwatch-statistics',
    keyTypes: cloudwatchStatisticsMetric,
  });

  return metricKeyTypes;
}

export function generateMetricDefinition({
  id,
  db_id,
  metric_kind,
  metadata,
}: {
  id: string;
  db_id?: string;
  metric_kind: string;
  metadata: any;
}) {
  return {
    kind: 'Metric',
    id,
    db_id,
    metric_kind: metric_kind,
    metadata,
  } as MetricDefinition;
}
