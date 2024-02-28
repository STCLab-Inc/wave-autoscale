export interface MetricsDataStats {
  metricId: string;
  timestamps: string[];
  timestampFrequency: { x: string; y: number }[];
  numberOfValues: number;
  lastValues: string;
}

export interface MetricsData {
  name?: string;
  tags?: any;
  value: number;
  timestamp: number | string;
}

export type MetricsDataItem = {
  name?: string;
  tags?: any;
  values: MetricsData[];
};

export type MetricsDataItems = MetricsDataItem[];
