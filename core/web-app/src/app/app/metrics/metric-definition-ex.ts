import { MetricDefinition } from '@/types/bindings/metric-definition';

export interface MetricDefinitionEx extends MetricDefinition {
  isChecked: boolean;
  created_at: string;
  updated_at: string;
}
