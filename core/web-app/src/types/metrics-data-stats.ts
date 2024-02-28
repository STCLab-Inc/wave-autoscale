export interface MetricsDataStats {
  metricId: string;
  timestamps: string[];
  timestampFrequency: { x: string; y: number }[];
  numberOfValues: number;
  lastValues: string;
}
