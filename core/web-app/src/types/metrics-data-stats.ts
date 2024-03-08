// MetricsDataStatsMap is from /api/metrics-data/stats
export interface MetricsDataStatsMap {
  // lastValues is a JSON string including metrics data
  [metricId: string]: [timestamps: number[], lastValues: string];
}

export interface MetricsDataStats {
  metricId: string;
  timestamps: number[];
  countPerMinute: { minute: string; count: number }[];
  numberOfValues: number;
  // lastValues is a JSON string including metrics data
  lastValues: string;
}

export interface MetricsDataValue {
  name?: string;
  tags?: any;
  value: number;
  timestamp: number | string;
}

export type MetricsDataItem = {
  name?: string;
  tags?: any;
  values: MetricsDataValue[];
};

export type MetricsDataItems = MetricsDataItem[];
