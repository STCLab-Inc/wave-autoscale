import { AutoscalingHistoryDefinition } from '@/types/bindings/autoscaling-history-definition';

export interface AutoscalingHistoryDefinitionEx
  extends AutoscalingHistoryDefinition {
  created_at: number;
}
