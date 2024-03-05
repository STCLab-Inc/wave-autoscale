import { ScalingPlanDefinition } from '@/types/bindings/scaling-plan-definition';
import { DailyEvent } from './plan-log-stats';

export interface ScalingPlanDefinitionEx extends ScalingPlanDefinition {
  metadata: { cool_down: number; interval: number; title: string };
  yaml?: string;
}

export type ScalingPlanWithStats = ScalingPlanDefinitionEx & {
  dailyStats: DailyEvent[];
};
