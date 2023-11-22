import { CloudwatchDataMetricMetadata } from '@/types/bindings/cloudwatch-data-metric';
import { CloudwatchStatisticsMetricMetadata } from '@/types/bindings/cloudwatch-statistics-metric';
import { MetricDefinition } from '@/types/bindings/metric-definition';
import { PrometheusMetricMetadata } from '@/types/bindings/prometheus-metric';
import { getInterfaceKeyTypes, MetadataKeyType } from './metadata-key-type';

export type MetricKeyType = { metricName: string; keyTypes: MetadataKeyType[] };

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
  kind,
  id,
  db_id,
  collector,
  metadata,
}: {
  kind: string;
  id: string;
  db_id?: string;
  collector: string;
  metadata: any;
}) {
  return {
    kind: kind,
    id,
    db_id,
    collector,
    metadata,
  } as MetricDefinition;
}
