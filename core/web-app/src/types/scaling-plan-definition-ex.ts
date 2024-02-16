import { ScalingPlanDefinition } from '@/types/bindings/scaling-plan-definition';

export interface ScalingPlanDefinitionEx extends ScalingPlanDefinition {
  metadata: { cool_down: number; interval: number; title: string };
  yaml?: string;
}
