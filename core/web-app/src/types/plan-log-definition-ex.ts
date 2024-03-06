import { PlanLogDefinition } from '@/types/bindings/plan-log-definition';

export interface PlanLogDefinitionEx extends PlanLogDefinition {
  // Unix timestamp in milliseconds
  created_at: number;
}
