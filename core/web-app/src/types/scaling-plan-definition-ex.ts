import { ScalingPlanDefinition } from '@/types/bindings/scaling-plan-definition';
import { PlanLogCountPerDay } from './plan-log-stats';

export interface ScalingPlanDefinitionEx extends ScalingPlanDefinition {
  metadata: { cool_down: number; interval: number; title: string };
  yaml?: string;
}

export type ScalingPlanWithStats = ScalingPlanDefinitionEx & {
  countPerDay: PlanLogCountPerDay[];
};
