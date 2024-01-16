import { CloudwatchDataMetricMetadata } from '@/types/bindings/cloudwatch-data-metric';
import { CloudwatchStatisticsMetricMetadata } from '@/types/bindings/cloudwatch-statistics-metric';
import { MetricDefinition } from '@/types/bindings/metric-definition';
import { PrometheusMetricMetadata } from '@/types/bindings/prometheus-metric';
import { getInterfaceKeyTypes, MetadataKeyType } from './metadata-key-type';
import JSYaml from 'js-yaml';

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
  enabled,
  metadata,
}: {
  kind: string;
  id: string;
  db_id?: string;
  collector: string;
  enabled: boolean;
  metadata: any;
}) {
  return {
    kind: kind,
    id,
    db_id,
    collector,
    enabled,
    metadata: metadata ?? {},
  } as MetricDefinition;
}

export function serializeMetricDefinition(
  metricDefinition: MetricDefinition
): string {
  const { kind, id, collector, enabled, metadata } = metricDefinition;
  const serialized = JSYaml.dump({
    kind,
    id,
    collector,
    enabled,
    metadata,
  });
  return serialized;
}

export function serializeMetricDefinitions(
  metricDefinitions: MetricDefinition[]
): string {
  const serialized = metricDefinitions.map((metricDefinition) =>
    serializeMetricDefinition(metricDefinition)
  );
  const result = serialized.join('\n---\n');
  return result;
}

export function deserializeMetricDefinitions(
  serialized: string
): MetricDefinition[] {
  let deserialized: MetricDefinition[] = JSYaml.loadAll(
    serialized
  ) as MetricDefinition[];
  // Filter out invalid component definitions
  deserialized = deserialized.filter(
    (definition) => definition !== null
  );
  return deserialized;
}
